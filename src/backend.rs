use std::collections::hash_map::Entry::Vacant;
use std::thread;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    runtime::Builder,
    select, spawn,
    sync::mpsc::{Receiver, Sender, channel},
};
use tokio_util::sync::CancellationToken;

use crate::fetcher::build_fetcher;
use crate::{Sites, Strip};

type Fetcher = Arc<dyn crate::Fetcher + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum RequestStripType {
    Last,
    #[default]
    Random,
    Next(Option<usize>),
    Prev(Option<usize>),
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Request {
    Strip { site: Sites, ty: RequestStripType },
    Download { path: PathBuf, url: String },
}

#[derive(Debug)]
pub enum Response {
    Strip(Option<Strip>),
    Download(Result<()>),
}

pub fn start_backend() -> (Handle, Sender<Request>, Receiver<Response>) {
    let (input, input_receiver) = channel(60);
    let (output_sender, output) = channel(60);
    let rt = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());
    let rt_handle = rt.clone();
    rt.spawn(async move {
        let rt = rt_handle;
        background_task(input_receiver, output_sender).await;

        // Drop the runtime from sync context to avoid panicking on exit
        thread::spawn(move || {
            let _rt = rt;
        });
    });
    (rt.handle().clone(), input, output)
}

async fn background_task(mut rx: Receiver<Request>, tx: Sender<Response>) {
    let mut fetchers: HashMap<Sites, Fetcher> = HashMap::default();
    let mut cancel_token = None;

    while let Some(req) = rx.recv().await {
        match req {
            Request::Strip { site, ty } => {
                if let Vacant(e) = fetchers.entry(site)
                    && let Some(val) = build_fetcher(site).await.map(|f| Arc::new(f) as Fetcher)
                {
                    e.insert(val);
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
