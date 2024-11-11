use std::fmt::Display;

use anyhow::Result;
use async_trait::async_trait;
use strum_macros::EnumIter;
use thiserror::Error;

pub mod backend;
pub mod fetcher;
pub mod frontend;

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
    CadComics,
    JoyOfTech,
    GoodTechThings,
    ThreeWordPhrase,
    ASofterWorld,
    ButterSafe,
    QuestionableContent,
    WorkChronicles,
    JSPowerHour,
    ButtercupFestival,
    Achewood,
    CatAndGirl,
    DieselSweeties1_0,
    DieselSweeties3_0,
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
    async fn next(&self, idx: usize) -> Result<Strip>;
    async fn prev(&self, idx: usize) -> Result<Strip>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripType {
    First,
    Unknown,
    Last,
    Unique,
}

#[derive(Debug, Clone)]
pub struct Strip {
    pub title: String,
    pub url: String,
    pub idx: usize,
    strip_type: StripType,
    pub site: Sites,
}

impl Strip {
    pub fn has_next(&self) -> bool {
        self.strip_type != StripType::First && self.strip_type != StripType::Unique
    }

    pub fn has_prev(&self) -> bool {
        self.strip_type != StripType::Last && self.strip_type != StripType::Unique
    }

    pub fn file_name(&self) -> String {
        let ext = self.url.split('.').rev().take(1).next().unwrap_or_default();
        format!("{}.{ext}", self.title)
    }
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
            Sites::CadComics => "https://cad-comic.com/feed",
            Sites::JoyOfTech => "https://www.joyoftech.com/joyoftech/jotblog",
            // Incomplete RSS feed.
            Sites::GoodTechThings => "https://www.goodtechthings.com/rss/",
            Sites::ThreeWordPhrase => "https://threewordphrase.com/archive.htm",
            Sites::ASofterWorld => "https://www.asofterworld.com/archive.php",
            Sites::ButterSafe => "https://www.buttersafe.com/archive",
            Sites::QuestionableContent => "https://www.questionablecontent.net/QCRSS.xml",
            Sites::WorkChronicles => "https://workchronicles.substack.com",
            Sites::JSPowerHour => "https://www.jspowerhour.com/comics",
            Sites::ButtercupFestival => "https://www.buttercupfestival.com",
            Sites::Achewood => "https://achewood.com/archive_new.html",
            Sites::CatAndGirl => "https://catandgirl.com/archive",
            Sites::DieselSweeties1_0 => "https://www.dieselsweeties.com/archive",
            Sites::DieselSweeties3_0 => "https://www.dieselsweeties.com/ds-unifeed.xml",
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
            Sites::CadComics => "cad-comic.com",
            Sites::JoyOfTech => "joyoftech.com/joyoftech",
            Sites::GoodTechThings => "goodtechthings.com",
            Sites::ThreeWordPhrase => "threewordphrase.com",
            Sites::ASofterWorld => "asofterworld.com",
            Sites::ButterSafe => "buttersafe.com",
            Sites::QuestionableContent => "questionablecontent.net",
            Sites::WorkChronicles => "workchronicles.com",
            Sites::JSPowerHour => "jspowerhour.com",
            Sites::ButtercupFestival => "buttercupfestival.com",
            Sites::Achewood => "achewood.com",
            Sites::CatAndGirl => "catandgirl.com",
            Sites::DieselSweeties1_0 => "dieselsweeties.com",
            Sites::DieselSweeties3_0 => "dieselsweeties.com",
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
            Sites::Oglaf => "Oglaf [NSFW]",
            Sites::CadComics => "CTRL+ALT+DEL",
            Sites::JoyOfTech => "The Joy of Tech",
            Sites::GoodTechThings => "Good Tech Things",
            Sites::ThreeWordPhrase => "Three Word Phrase",
            Sites::ASofterWorld => "a softer world",
            Sites::ButterSafe => "BUTTERSAFE",
            Sites::QuestionableContent => "Questionable Content",
            Sites::WorkChronicles => "Work Chronicles",
            Sites::JSPowerHour => "Junior Scientist Power Hour",
            Sites::ButtercupFestival => "Buttercup Festival",
            Sites::Achewood => "achewood",
            Sites::CatAndGirl => "Cat and Girl",
            Sites::DieselSweeties1_0 => "Diesel Sweeties #1.0",
            Sites::DieselSweeties3_0 => "Diesel Sweeties #3.0",
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
    async fn test_monkey_user() {
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
        let fetcher = build_fetcher(crate::Sites::CadComics).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    // Fails on gh ci
    #[ignore]
    #[tokio::test]
    async fn test_joy_of_tech() {
        let fetcher = build_fetcher(crate::Sites::JoyOfTech).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_good_tech_things() {
        let fetcher = build_fetcher(crate::Sites::GoodTechThings).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_three_word_phrase() {
        let fetcher = build_fetcher(crate::Sites::ThreeWordPhrase).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_a_softer_world() {
        let fetcher = build_fetcher(crate::Sites::ASofterWorld).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_butter_safe() {
        let fetcher = build_fetcher(crate::Sites::ButterSafe).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_questionable_content() {
        let fetcher = build_fetcher(crate::Sites::QuestionableContent).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    // Fails on gh ci
    #[ignore]
    #[tokio::test]
    async fn test_work_chronicles() {
        let fetcher = build_fetcher(crate::Sites::WorkChronicles).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_js_power_hour() {
        let fetcher = build_fetcher(crate::Sites::JSPowerHour).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_buttercup_festival() {
        let fetcher = build_fetcher(crate::Sites::ButtercupFestival).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_achewood() {
        let fetcher = build_fetcher(crate::Sites::Achewood).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_cat_and_girl() {
        let fetcher = build_fetcher(crate::Sites::CatAndGirl).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_diesel_sweeties_1_0() {
        let fetcher = build_fetcher(crate::Sites::DieselSweeties1_0).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        println!("{:?}", fetcher.last().await);
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_diesel_sweeties_3_0() {
        let fetcher = build_fetcher(crate::Sites::DieselSweeties3_0).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        println!("{:?}", fetcher.last().await);
        assert!(fetcher.random().await.is_ok());
    }
}
