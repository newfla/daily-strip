use std::fmt::Display;

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;

mod fetcher;
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
}

#[async_trait]
pub trait Fetcher {
    async fn reload(&mut self) -> Result<()>;
    async fn last(&mut self) -> Result<Strip>;
    async fn random(&mut self) -> Result<Strip>;
}

pub type Strip = (String, Vec<u8>);

#[derive(Error, Debug)]
pub enum FetcherErrors {
    #[error("Failed to fetch items")]
    Error404,
}

trait ToUrl {
    fn to_url(&self) -> String;
}

impl ToUrl for Sites {
    fn to_url(&self) -> String {
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
        }
        .to_string()
    }
}

impl Display for Sites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_url())
    }
}

#[cfg(test)]
mod test {
    use crate::{fetcher::build_fetcher, Fetcher};

    #[tokio::test]
    async fn test_turnoff_us() {
        let fetcher = build_fetcher(crate::Sites::TurnoffUs).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_monkey_user() {
        let fetcher = build_fetcher(crate::Sites::MonkeyUser).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_bonkers_world() {
        let fetcher = build_fetcher(crate::Sites::BonkersWorld).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_goomics() {
        let fetcher = build_fetcher(crate::Sites::Goomics).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_xkcd() {
        let fetcher = build_fetcher(crate::Sites::Xkcd).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_oglaf() {
        let fetcher = build_fetcher(crate::Sites::Oglaf).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_dinosaur_comics() {
        let fetcher = build_fetcher(crate::Sites::DinosaurComics).await;
        assert!(fetcher.is_some());
        let mut fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }
}
