use anyhow::Result;
use cached::proc_macro::cached;
use cached::SizedCache;
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
    sync::mpsc::{channel, Receiver, Sender},
};

type Fetcher = Arc<dyn daily_strip::Fetcher + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum RequestType {
    Last,
    Random,
    Next(Option<usize>),
    Prev(Option<usize>),
}

impl Default for RequestType {
    fn default() -> Self {
        Self::Random
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum Request {
    Strip { site: Sites, ty: RequestType },
    Download { path: PathBuf, url: String },
}

enum Response {
    Strip(Option<Strip>),
    Download(Result<()>),
}

fn main() -> Result<(), eframe::Error> {
    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };

    let (tx_req, rx_req) = channel(60);
    let (tx_res, rx_res) = channel(60);

    let app = App {
        mode: RequestType::Last,
        source: Sites::default(),
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
    mode: RequestType,
    source: Sites,
    tx: Sender<Request>,
    rx: Receiver<Response>,
    rt: Runtime,
}

impl App {
    fn start_background_task(&self, rx: Receiver<Request>, tx: Sender<Response>) {
        self.rt.spawn(async move { background_task(rx, tx).await });
    }

    fn get_content(&mut self) -> Option<Strip> {
        get_content_cached(
            Request::Strip {
                site: self.source,
                ty: self.mode,
            },
            &self.tx,
            &mut self.rx,
        )
    }

    fn maybe_download_content(
        &mut self,
        strip: &Strip,
        ctx: &eframe::egui::Context,
    ) -> Option<Result<()>> {
        if let Some(file_dialog) = self.file_dialog.as_mut() {
            if let Some(path) = file_dialog.update(ctx).selected() {
                let path = path.to_path_buf();
                let url = strip.url.clone();

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

    fn open_file_dialog(&mut self, strip: &Strip) {
        let file_name = strip.file_name();

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

#[cached(
    ty = "SizedCache<Request, Strip>",
    create = "{ SizedCache::with_size(10) }",
    convert = r#"{req.clone()}"#,
    sync_writes = true,
    option = true
)]

fn get_content_cached(
    req: Request,
    tx: &Sender<Request>,
    rx: &mut Receiver<Response>,
) -> Option<Strip> {
    let _ = tx.blocking_send(req);
    rx.blocking_recv().map(|elem| {
        if let Response::Strip(strip) = elem {
            strip
        } else {
            None
        }
    })?
}

fn force_refresh() {
    let mut lock = GET_CONTENT_CACHED.lock().unwrap();
    *lock = SizedCache::with_size(10);
}

async fn background_task(mut rx: Receiver<Request>, tx: Sender<Response>) {
    let mut fetchers: HashMap<Sites, Option<Fetcher>> = HashMap::default();

    while let Some(req) = rx.recv().await {
        match req {
            Request::Strip { site, ty } => {
                let content = get_content_background(site, ty, &mut fetchers).await;
                let _ = tx.send(Response::Strip(content)).await;
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

async fn get_content_background(
    site: Sites,
    ty: RequestType,
    fetchers: &mut HashMap<Sites, Option<Fetcher>>,
) -> Option<Strip> {
    if let Vacant(e) = fetchers.entry(site) {
        e.insert(build_fetcher(site).await.map(|f| Arc::new(f) as Fetcher));
    }
    if let Some(Some(fetcher)) = fetchers.get(&site) {
        return match ty {
            RequestType::Last => fetcher.last().await.ok(),
            RequestType::Random => fetcher.random().await.ok(),
            RequestType::Next(Some(idx)) => fetcher.next(idx).await.ok(),
            RequestType::Prev(Some(idx)) => fetcher.prev(idx).await.ok(),
            _ => None,
        };
    }
    None
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let strip = self.get_content();
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
                                self.mode = RequestType::Last;
                            }
                        }
                    });

                let homepage = self.source.homepage();
                ui.hyperlink_to(homepage, "https://".to_owned() + homepage);

                ui.separator();

                let prev_available = strip.as_ref().map(Strip::has_prev).unwrap_or(false);
                let next_available = strip.as_ref().map(Strip::has_next).unwrap_or(false);

                ui.add_enabled_ui(prev_available, |ui| {
                    if ui.button("Prev").clicked() {
                        self.mode = RequestType::Prev(strip.as_ref().map(|s| s.idx));
                        force_refresh()
                    }
                });

                ui.add_enabled_ui(next_available, |ui| {
                    if ui.button("Next").clicked() {
                        self.mode = RequestType::Next(strip.as_ref().map(|s| s.idx));
                        force_refresh()
                    }
                });

                ui.add_enabled_ui(next_available || strip.is_none(), |ui| {
                    if ui.button("Last").clicked() {
                        self.mode = RequestType::Last;
                        force_refresh()
                    }
                });

                if ui.button("Random").clicked() {
                    self.mode = RequestType::Random;
                    force_refresh()
                }

                if let Some(content) = strip.as_ref() {
                    ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                        ui.add(Label::new(&content.title).truncate());

                        ui.separator();

                        if ui.button("Download").clicked() {
                            self.open_file_dialog(content);
                        }
                    });
                    self.maybe_download_content(content, ctx);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(eframe::egui::Direction::LeftToRight),
                |ui| {
                    if let Some(content) = strip.as_ref() {
                        ui.image(&content.url);
                    }
                },
            )
        });
    }
}
