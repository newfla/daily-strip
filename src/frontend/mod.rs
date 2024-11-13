use anyhow::Result;
use tokio::{
    runtime::Handle,
    sync::mpsc::{Receiver, Sender},
};

use crate::backend::{Request, Response};

#[cfg(feature = "egui_frontend")]
pub mod egui;

#[cfg(feature = "slint_frontend")]
pub mod slint;

pub trait Runnable {
    fn run(handle: Handle, tx: Sender<Request>, rx: Receiver<Response>) -> Result<()>;
}
