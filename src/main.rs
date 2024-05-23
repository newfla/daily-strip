use cached::proc_macro::cached;
use cached::SizedCache;
use daily_strip::{build_fetcher, Sites, Strip, Url};
use eframe::egui::{
    ahash::HashMap, CentralPanel, ComboBox, Layout, TopBottomPanel, ViewportBuilder,
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

    let _ = tx_req.blocking_send(Request::default());
    app.start_background_task(rx_req, tx_res);

    eframe::run_native(
        "Daily Strip",
        opts,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(app)
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
}

async fn background_task(mut rx: Receiver<Request>, tx: Sender<Option<Strip>>) {
    let mut fetchers: HashMap<Sites, Option<Fetcher>> = HashMap::default();

    while let Some(req) = rx.recv().await {
        let content = get_content(req, &mut fetchers).await;
        let _ = tx.send(content).await;
    }
}

async fn get_content(
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
    if let Some(f) = fetchers.get(&req.site).unwrap().as_ref() {
        return cached_content(req, f).await;
    }
    None
}

#[cached(
    ty = "SizedCache<Request, Option<Strip>>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{req}"#,
    sync_writes = true
)]
async fn cached_content(req: Request, fetcher: &Fetcher) -> Option<Strip> {
    match req.ty {
        RequestType::Last => fetcher.last().await.ok(),
        RequestType::Random => fetcher.random().await.ok(),
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let strip = self.rx.blocking_recv().flatten();
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
                ui.hyperlink_to(&homepage, "https://www.".to_owned() + &homepage);
                ui.radio_value(
                    &mut self.mode,
                    RequestType::Last,
                    format!("{:?}", RequestType::Last),
                );
                ui.radio_value(
                    &mut self.mode,
                    RequestType::Random,
                    format!("{:?}", RequestType::Random),
                );
                if let Some(content) = strip.as_ref() {
                    ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                        ui.label("Showing: ".to_owned() + &content.title)
                    });
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            let _ = self.tx.blocking_send(Request {
                site: self.source,
                ty: self.mode,
            });

            if let Some(content) = strip.as_ref() {
                println!("{content:?}");
                ui.image(&content.url);
            }
        });
    }
}
