use std::fmt::Display;

use anyhow::Result;
use async_trait::async_trait;
use strum_macros::EnumIter;
use thiserror::Error;

mod fetcher;
pub use fetcher::build_fetcher;

#[derive(Debug, Clone, Copy, EnumIter, Hash, PartialEq, Eq)]
/// Supported strip sites
#[non_exhaustive]
pub enum Sites {
    TurnoffUs,
    MonkeyUser,
    BonkersWorld,
    Goomics,
    Xkcd,
    DinosaurComics,
    Oglaf,
    Cad,
}

impl Default for Sites {
    fn default() -> Self {
        Self::TurnoffUs
    }
}

#[async_trait]
pub trait Fetcher {
    async fn reload(&mut self) -> Result<()>;
    async fn last(&self) -> Result<Strip>;
    async fn random(&self) -> Result<Strip>;
}

#[derive(Debug, Clone)]
pub struct Strip {
    pub title: String,
    pub url: String,
}

#[derive(Error, Debug)]
pub enum FetcherErrors {
    #[error("Failed to fetch items")]
    Error404,
}

pub trait Url {
    fn fetch_url(&self) -> &str;
    fn homepage(&self) -> &str;
}

impl Url for Sites {
    fn fetch_url(&self) -> &str {
        match self {
            // Incomplete RSS feed. Switching to scraping
            Sites::TurnoffUs => "https://turnoff.us",
            Sites::MonkeyUser => "https://www.monkeyuser.com/index.xml",
            // Incomplete RSS feed. Switching to scraping
            Sites::BonkersWorld => "https://bonkersworld.net",
            // Incomplete RSS feed. Switching to scraping
            Sites::Goomics => "https://goomics.net",
            // Incomplete RSS feed. Switching to scraping
            Sites::Xkcd => "https://xkcd.com",
            // Incomplete RSS feed. Switching to scraping
            Sites::DinosaurComics => "https://www.qwantz.com",
            Sites::Oglaf => "https://www.oglaf.com/feeds/rss",
            // Incomplete RSS feed.
            Self::Cad => "https://cad-comic.com/feed",
        }
    }

    fn homepage(&self) -> &str {
        match self {
            Sites::TurnoffUs => "turnoff.us",
            Sites::MonkeyUser => "monkeyuser.com",
            Sites::BonkersWorld => "bonkersworld.net",
            Sites::Goomics => "goomics.net",
            Sites::Xkcd => "xkcd.com",
            Sites::DinosaurComics => "qwantz.com",
            Sites::Oglaf => "oglaf.com",
            Sites::Cad => "cad-comic.com",
        }
    }
}

impl Display for Sites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Sites::TurnoffUs => "{turnoff.us}",
            Sites::MonkeyUser => "MonkeyUser",
            Sites::BonkersWorld => "Bonkers Worls",
            Sites::Goomics => "Goomics",
            Sites::Xkcd => "xkcd",
            Sites::DinosaurComics => "Dinosaur Comics",
            Sites::Oglaf => "Oglaf.com",
            Sites::Cad => "CTRL+ALT+DEL",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod test {
    use crate::{fetcher::build_fetcher, Fetcher};

    #[tokio::test]
    async fn test_turnoff_us() {
        let fetcher = build_fetcher(crate::Sites::TurnoffUs).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_monkeyuser() {
        let fetcher = build_fetcher(crate::Sites::MonkeyUser).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_bonkers_world() {
        let fetcher = build_fetcher(crate::Sites::BonkersWorld).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_goomics() {
        let fetcher = build_fetcher(crate::Sites::Goomics).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_xkcd() {
        let fetcher = build_fetcher(crate::Sites::Xkcd).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_oglaf() {
        let fetcher = build_fetcher(crate::Sites::Oglaf).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_dinosaur_comics() {
        let fetcher = build_fetcher(crate::Sites::DinosaurComics).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_cmd() {
        let fetcher = build_fetcher(crate::Sites::Cad).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }
}
