use anyhow::Result;
use daily_strip::{build_fetcher, Sites, Strip, Url};
use eframe::egui::{
    ahash::HashMap, CentralPanel, ComboBox, Label, Layout, TopBottomPanel, ViewportBuilder,
};
use egui_file_dialog::FileDialog;
use std::{collections::hash_map::Entry::Vacant, hash::Hash, path::PathBuf, sync::Arc};
use strum::IntoEnumIterator;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    runtime::{Builder, Runtime},
    select, spawn,
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio_util::sync::CancellationToken;

type Fetcher = Arc<dyn daily_strip::Fetcher + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum RequestStripType {
    Last,
    Random,
    Next(Option<usize>),
    Prev(Option<usize>),
}

impl Default for RequestStripType {
    fn default() -> Self {
        Self::Random
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum Request {
    Strip { site: Sites, ty: RequestStripType },
    Download { path: PathBuf, url: String },
}

enum Response {
    Strip(Option<Strip>),
    Download(Result<()>),
}

fn main() -> Result<(), eframe::Error> {
    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1024.0, 1024.0]),
        ..Default::default()
    };

    let (tx_req, rx_req) = channel(60);
    let (tx_res, rx_res) = channel(60);

    let app = App {
        mode: RequestStripType::Last,
        source: Sites::default(),
        strip: None,
        tx: tx_req.clone(),
        rx: rx_res,
        rt: Builder::new_multi_thread().enable_all().build().unwrap(),
        file_dialog: Some(FileDialog::new()),
    };

    app.start_background_task(rx_req, tx_res);

    eframe::run_native(
        "Daily Strip",
        opts,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
struct App {
    file_dialog: Option<FileDialog>,
    mode: RequestStripType,
    source: Sites,
    strip: Option<Option<Strip>>,
    tx: Sender<Request>,
    rx: Receiver<Response>,
    rt: Runtime,
}

impl App {
    fn start_background_task(&self, rx: Receiver<Request>, tx: Sender<Response>) {
        self.rt.spawn(async move { background_task(rx, tx).await });
    }

    fn force_refresh(&mut self, mode: RequestStripType) {
        self.strip = None;
        self.mode = mode;
    }

    fn get_content(&mut self) -> &Option<Strip> {
        match self.strip {
            None => {
                let req = Request::Strip {
                    site: self.source,
                    ty: self.mode,
                };
                if self.tx.blocking_send(req).is_ok() {
                    self.strip = Some(None);
                }
                &None
            }

            Some(None) => match self.rx.try_recv() {
                Ok(Response::Strip(data)) => {
                    if data.as_ref().is_some_and(|data| data.site == self.source) {
                        self.strip = Some(data);
                        self.strip.as_ref().unwrap()
                    } else {
                        &None
                    }
                }
                _ => &None,
            },
            Some(ref val) => val,
        }
    }

    fn maybe_download_content(
        &mut self,
        url: String,
        ctx: &eframe::egui::Context,
    ) -> Option<Result<()>> {
        if let Some(file_dialog) = self.file_dialog.as_mut() {
            if let Some(path) = file_dialog.update(ctx).selected() {
                let path = path.to_path_buf();

                self.file_dialog = None;

                let _ = self.tx.blocking_send(Request::Download { path, url });
                return if let Some(Response::Download(res)) = self.rx.blocking_recv() {
                    Some(res)
                } else {
                    None
                };
            }
        }
        None
    }

    fn open_file_dialog(&mut self, file_name: String) {
        // Workaround for state never be Closed
        if self.file_dialog.is_none() {
            self.file_dialog = Some(FileDialog::new())
        }

        if let Some(file_dialog) = self.file_dialog.as_mut() {
            file_dialog.config_mut().default_file_name = file_name;
            file_dialog.save_file();
        }
    }
}
async fn background_task(mut rx: Receiver<Request>, tx: Sender<Response>) {
    let mut fetchers: HashMap<Sites, Fetcher> = HashMap::default();
    let mut cancel_token = None;

    while let Some(req) = rx.recv().await {
        match req {
            Request::Strip { site, ty } => {
                if let Vacant(e) = fetchers.entry(site) {
                    if let Some(val) = build_fetcher(site).await.map(|f| Arc::new(f) as Fetcher) {
                        e.insert(val);
                    }
                }

                let tx = tx.clone();

                let fetcher = fetchers.get(&site).unwrap().clone();
                let actual_cancel_token = CancellationToken::new();
                if let Some(prev_token) = cancel_token.replace(actual_cancel_token.clone()) {
                    prev_token.cancel();
                }
                spawn(async move {
                    select! {
                        _ = actual_cancel_token.cancelled() => {}
                        content = get_content_background(ty, fetcher) => {
                            let _ = tx.send(Response::Strip(content)).await;

                        }
                    }
                });
            }
            Request::Download { path, url } => {
                let res = download_background(path, url).await;
                let _ = tx.send(Response::Download(res)).await;
            }
        }
    }
}

async fn download_background(path: PathBuf, url: String) -> Result<()> {
    let data = reqwest::get(url).await?.bytes().await?;
    let mut file = File::create(path).await?;
    file.write_all(&data).await?;
    Ok(())
}

async fn get_content_background(ty: RequestStripType, fetcher: Fetcher) -> Option<Strip> {
    match ty {
        RequestStripType::Last => fetcher.last().await.ok(),
        RequestStripType::Random => fetcher.random().await.ok(),
        RequestStripType::Next(Some(idx)) => fetcher.next(idx).await.ok(),
        RequestStripType::Prev(Some(idx)) => fetcher.prev(idx).await.ok(),
        _ => None,
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut sites: Vec<_> = Sites::iter().collect();
        sites.sort_by_key(|s| format!("{s}").to_lowercase());
        TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.source))
                    .show_ui(ui, |ui| {
                        for site in sites.into_iter() {
                            if ui
                                .selectable_value(&mut self.source, site, format!("{}", site))
                                .changed()
                            {
                                self.force_refresh(RequestStripType::Last);
                            }
                        }
                    });

                let homepage = self.source.homepage();
                ui.hyperlink_to(homepage, "https://".to_owned() + homepage);

                ui.separator();

                let (prev_available, next_available) = {
                    let strip = self.get_content().as_ref();
                    let prev_available = strip.map(Strip::has_prev).unwrap_or(false);
                    let next_available = strip.map(Strip::has_next).unwrap_or(false);
                    (prev_available, next_available)
                };

                ui.add_enabled_ui(prev_available, |ui| {
                    if ui.button("Prev").clicked() {
                        let mode =
                            RequestStripType::Prev(self.get_content().as_ref().map(|s| s.idx));
                        self.force_refresh(mode);
                    }
                });

                ui.add_enabled_ui(next_available, |ui| {
                    if ui.button("Next").clicked() {
                        let mode =
                            RequestStripType::Next(self.get_content().as_ref().map(|s| s.idx));
                        self.force_refresh(mode)
                    }
                });

                ui.add_enabled_ui(
                    next_available || self.get_content().as_ref().is_none(),
                    |ui| {
                        if ui.button("Last").clicked() {
                            self.force_refresh(RequestStripType::Last)
                        }
                    },
                );

                if ui.button("Random").clicked() {
                    self.force_refresh(RequestStripType::Random)
                }

                if let Some((title, url, file_name)) = self
                    .get_content()
                    .as_ref()
                    .map(|strip| (strip.title.clone(), strip.url.clone(), strip.file_name()))
                {
                    ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                        ui.add(Label::new(&title).truncate());

                        ui.separator();

                        if ui.button("Download").clicked() {
                            self.open_file_dialog(file_name);
                        }
                    });
                    self.maybe_download_content(url, ctx);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(eframe::egui::Direction::LeftToRight),
                |ui| match self.get_content() {
                    None => ui.spinner(),
                    Some(content) => ui.image(&content.url),
                },
            )
        });
    }
}
