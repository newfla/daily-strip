use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend::{Request, Response};

#[cfg(feature = "egui")]
pub mod egui;

pub trait Runnable {
    fn run(tx: Sender<Request>, rx: Receiver<Response>) -> Result<()>;
}
