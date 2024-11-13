use anyhow::{anyhow, Result};
use eframe::egui::{CentralPanel, ComboBox, Label, Layout, TopBottomPanel, ViewportBuilder};
use egui_file_dialog::FileDialog;
use egui_theme_switcher::theme_switcher;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    backend::{Request, RequestStripType, Response},
    Sites, Strip, Url,
};

use super::Runnable;

#[derive(Default)]
pub struct EguiFrontend;

impl Runnable for EguiFrontend {
    fn run(_handle: Handle, tx: Sender<Request>, rx: Receiver<Response>) -> Result<()> {
        let opts = eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size([1024.0, 1024.0]),
            ..Default::default()
        };

        let app = App {
            mode: RequestStripType::Last,
            source: Sites::default(),
            strip: None,
            tx,
            rx,
            file_dialog: Some(FileDialog::new()),
        };

        eframe::run_native(
            "Daily Strip",
            opts,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Ok(Box::new(app))
            }),
        )
        .map_err(|err| anyhow!(err.to_string()))
    }
}

struct App {
    file_dialog: Option<FileDialog>,
    mode: RequestStripType,
    source: Sites,
    strip: Option<Option<Strip>>,
    tx: Sender<Request>,
    rx: Receiver<Response>,
}

impl App {
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

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let sites: Vec<_> = Sites::sites_sorted();
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

                ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                    ui.add(theme_switcher());
                    ui.separator();

                    if let Some((title, url, file_name)) = self
                        .get_content()
                        .as_ref()
                        .map(|strip| (strip.title.clone(), strip.url.clone(), strip.file_name()))
                    {
                        if ui.button("Download").clicked() {
                            self.open_file_dialog(file_name);
                        }

                        ui.add(Label::new(&title).truncate());

                        self.maybe_download_content(url, ctx);
                    }
                });
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
