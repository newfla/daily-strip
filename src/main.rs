use cached::proc_macro::cached;
use cached::SizedCache;
use daily_strip::{build_fetcher, Sites, Strip, Url};
use eframe::egui::{
    ahash::HashMap, CentralPanel, ComboBox, Label, Layout, RadioButton, TopBottomPanel,
    ViewportBuilder,
};
use std::{collections::hash_map::Entry::Vacant, hash::Hash, sync::Arc};

use strum::IntoEnumIterator;
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{channel, Receiver, Sender},
};

type Fetcher = Arc<dyn daily_strip::Fetcher + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum RequestType {
    Last,
    Random,
}

impl Default for RequestType {
    fn default() -> Self {
        Self::Random
    }
}

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
struct Request {
    site: Sites,
    ty: RequestType,
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
    mode: RequestType,
    source: Sites,
    tx: Sender<Request>,
    rx: Receiver<Option<Strip>>,
    rt: Runtime,
}

impl App {
    fn start_background_task(&self, rx: Receiver<Request>, tx: Sender<Option<Strip>>) {
        self.rt.spawn(async move { background_task(rx, tx).await });
    }

    fn get_content(&mut self) -> Option<Strip> {
        get_content_cached(
            Request {
                site: self.source,
                ty: self.mode,
            },
            &self.tx,
            &mut self.rx,
        )
    }
}

#[cached(
    ty = "SizedCache<Request, Option<Strip>>",
    create = "{ SizedCache::with_size(10) }",
    convert = r#"{req}"#,
    sync_writes = true
)]

fn get_content_cached(
    req: Request,
    tx: &Sender<Request>,
    rx: &mut Receiver<Option<Strip>>,
) -> Option<Strip> {
    let _ = tx.blocking_send(req);
    rx.blocking_recv().flatten()
}

fn force_refresh() {
    let mut lock = GET_CONTENT_CACHED.lock().unwrap();
    *lock = SizedCache::with_size(10);
}

async fn background_task(mut rx: Receiver<Request>, tx: Sender<Option<Strip>>) {
    let mut fetchers: HashMap<Sites, Option<Fetcher>> = HashMap::default();

    while let Some(req) = rx.recv().await {
        let content = get_content_background(req, &mut fetchers).await;
        let _ = tx.send(content).await;
    }
}

async fn get_content_background(
    req: Request,
    fetchers: &mut HashMap<Sites, Option<Fetcher>>,
) -> Option<Strip> {
    if let Vacant(e) = fetchers.entry(req.site) {
        e.insert(
            build_fetcher(req.site)
                .await
                .map(|f| Arc::new(f) as Fetcher),
        );
    }
    if let Some(fetcher) = fetchers.get(&req.site).unwrap().as_ref() {
        return match req.ty {
            RequestType::Last => fetcher.last().await.ok(),
            RequestType::Random => fetcher.random().await.ok(),
        };
    }
    None
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let strip = self.get_content();
        TopBottomPanel::bottom("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.source))
                    .show_ui(ui, |ui| {
                        for site in Sites::iter() {
                            ui.selectable_value(&mut self.source, site, format!("{}", site));
                        }
                    });
                let homepage = self.source.homepage();
                ui.hyperlink_to(homepage, "https://".to_owned() + homepage);
                ui.radio_value(
                    &mut self.mode,
                    RequestType::Last,
                    format!("{:?}", RequestType::Last),
                );

                if ui
                    .add(RadioButton::new(
                        self.mode == RequestType::Random,
                        format!("{:?}", RequestType::Random),
                    ))
                    .clicked()
                {
                    self.mode = RequestType::Random;
                    force_refresh()
                }
                if ui.button("REFRESH").clicked() {
                    force_refresh()
                }
                if let Some(content) = strip.as_ref() {
                    ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                        ui.add(Label::new(&content.title).truncate())
                    });
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
