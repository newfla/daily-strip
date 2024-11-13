use anyhow::Result;
use daily_strip::backend::start_backend;
use daily_strip::frontend::Runnable;

fn main() -> Result<()> {
    let (handle, tx, rx) = start_backend();

    #[cfg(feature = "egui_frontend")]
    {
        daily_strip::frontend::egui::EguiFrontend::run(handle, tx, rx)
    }

    #[cfg(all(feature = "slint_frontend", not(feature = "egui_frontend")))]
    {
        daily_strip::frontend::slint::SlintFrontend::run(handle, tx, rx)
    }
}
