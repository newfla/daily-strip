// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use image::ImageReader;
use native_dialog::FileDialog;
use slint::{ComponentHandle, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Weak};
use std::{io::Cursor, str::FromStr};
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    backend::{Request, RequestStripType, Response},
    Sites, Url,
};

use super::Runnable;

slint::include_modules!();

#[derive(Default)]
pub struct SlintFrontend;

impl Runnable for SlintFrontend {
    fn run(handle: Handle, tx: Sender<Request>, rx: Receiver<Response>) -> Result<()> {
        let ui = AppWindow::new()?;
        // Setup ComboBox
        ui.set_sites(sites_to_model());

        let listener_ui_weak = ui.as_weak();
        let selected_ui_weak = ui.as_weak();
        let last_ui_weak = ui.as_weak();
        let random_ui_weak = ui.as_weak();
        let prev_ui_weak = ui.as_weak();
        let next_ui_weak = ui.as_weak();
        let download_ui_weak = ui.as_weak();

        let selected_tx = tx.clone();
        let next_tx = tx.clone();
        let prev_tx = tx.clone();
        let last_tx = tx.clone();
        let random_tx = tx.clone();
        let download_tx = tx.clone();

        ui.on_site_selected(move |site: SharedString| {
            // Will never explode. ComboBox values are derived by site.display()
            let site: Sites = Sites::from_str(&site).unwrap();
            let ui = selected_ui_weak.unwrap();

            ui.set_url_site(SharedString::from(site.homepage()));
            reset_strip(&ui);

            last(&selected_tx, site);
        });

        ui.on_open_url(move |site| {
            // Will never explode. ComboBox values are derived by site.display()
            let site = Sites::from_str(&site).unwrap();
            let url = format!("https://{}", site.homepage());
            let _ = open::that(&url);
        });

        ui.on_last(move |site| {
            let ui = last_ui_weak.unwrap();

            reset_strip(&ui);

            let site = Sites::from_str(&site).unwrap();
            last(&last_tx, site);
        });

        ui.on_random(move |site| {
            let ui = random_ui_weak.unwrap();

            reset_strip(&ui);

            let site = Sites::from_str(&site).unwrap();
            random(&random_tx, site);
        });

        ui.on_prev(move |site, idx| {
            let ui = prev_ui_weak.unwrap();

            reset_strip(&ui);

            let site = Sites::from_str(&site).unwrap();
            prev(&prev_tx, site, idx);
        });

        ui.on_next(move |site, idx| {
            let ui = next_ui_weak.unwrap();

            reset_strip(&ui);

            let site = Sites::from_str(&site).unwrap();
            next(&next_tx, site, idx);
        });

        ui.on_download(move |url, filename| {
            let ui = download_ui_weak.unwrap();
            if let Ok(Some(path)) = FileDialog::new()
                .set_filename(filename.as_str())
                .show_save_single_file()
            {
                let _ = download_tx.blocking_send(Request::Download {
                    path,
                    url: url.as_str().to_owned(),
                });
                let mut model = ui.get_strip();
                ui.set_comic_title_backup(model.title);
                model.title = SharedString::from("DOWNLOADING...");
                ui.set_strip(model);
            }
        });

        handle.spawn(async move {
            listener(rx, listener_ui_weak).await;
        });

        ui.invoke_site_selected(SharedString::from(Sites::ASofterWorld.to_string()));

        ui.run()?;
        Ok(())
    }
}

fn reset_strip(ui: &AppWindow) {
    let mut model = StripModel::default();
    model.title = SharedString::from("LOADING...");
    ui.set_strip(model);
    ui.set_loaded(false);
}
fn sites_to_model() -> ModelRc<SharedString> {
    let sites: Vec<_> = Sites::sites_sorted()
        .iter()
        .map(|site| SharedString::from(site.to_string()))
        .collect();
    ModelRc::from(sites.as_slice())
}

fn last(tx: &Sender<Request>, site: Sites) {
    let req = Request::Strip {
        site,
        ty: RequestStripType::Last,
    };
    let _ = tx.blocking_send(req);
}

fn random(tx: &Sender<Request>, site: Sites) {
    let req = Request::Strip {
        site,
        ty: RequestStripType::Random,
    };
    let _ = tx.blocking_send(req);
}

fn next(tx: &Sender<Request>, site: Sites, idx: i32) {
    let idx = idx as usize;
    let req = Request::Strip {
        site,
        ty: RequestStripType::Next(Some(idx)),
    };
    let _ = tx.blocking_send(req);
}

fn prev(tx: &Sender<Request>, site: Sites, idx: i32) {
    let idx = idx as usize;
    let req = Request::Strip {
        site,
        ty: RequestStripType::Prev(Some(idx)),
    };

    let _ = tx.blocking_send(req);
}

async fn listener(mut rx: Receiver<Response>, ui: Weak<AppWindow>) {
    while let Some(msg) = rx.recv().await {
        match msg {
            Response::Strip(Some(strip)) => {
                let buffer = load_image(&strip.url).await;
                let _ = ui.upgrade_in_event_loop(move |ui| {
                    if let Ok(buffer) = buffer {
                        ui.set_loaded(true);
                        let image = Image::from_rgba8(buffer);
                        let mut model = StripModel::default();
                        model.title = SharedString::from(&strip.title);
                        model.idx = strip.idx as i32;
                        model.image = image;
                        model.has_next = strip.has_next();
                        model.has_prev = strip.has_prev();
                        model.is_last = strip.is_last();
                        model.url = SharedString::from(&strip.url);
                        model.filename = SharedString::from(strip.file_name());
                        ui.set_strip(model);
                    }
                });
            }
            Response::Download(_) => {
                let _ = ui.upgrade_in_event_loop(|ui| {
                    let mut model = ui.get_strip();
                    model.title = ui.get_comic_title_backup();
                    ui.set_strip(model);
                });
            }
            _ => {}
        }
    }
}

async fn load_image(url: &str) -> Result<SharedPixelBuffer<Rgba8Pixel>> {
    let data = reqwest::get(url).await?.bytes().await?;

    let image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?
        .into_rgba8();
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        image.as_raw(),
        image.width(),
        image.height(),
    );
    Ok(buffer)
}
