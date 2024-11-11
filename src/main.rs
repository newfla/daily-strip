use anyhow::Result;
use daily_strip::backend::start_backend;
use daily_strip::frontend::egui::EguiFrontend;
use daily_strip::frontend::Runnable;

fn main() -> Result<()> {
    let (tx, rx) = start_backend();

    #[cfg(feature = "egui")]
    {
        EguiFrontend::run(tx, rx)
    }
}
