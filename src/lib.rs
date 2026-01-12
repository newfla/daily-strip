use anyhow::Result;
use async_trait::async_trait;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use thiserror::Error;

pub mod backend;
pub mod fetcher;
pub mod frontend;

#[derive(Debug, Default, Display, Clone, Copy, EnumIter, EnumString, Hash, PartialEq, Eq)]
/// Supported strip sites
#[non_exhaustive]
pub enum Sites {
    #[default]
    #[strum(to_string = "turnoff.us")]
    TurnoffUs,
    #[strum(to_string = "MonkeyUser")]
    MonkeyUser,
    #[strum(to_string = "Bonkers World")]
    BonkersWorld,
    #[strum(to_string = "Goomics")]
    Goomics,
    #[strum(to_string = "xkcd")]
    Xkcd,
    #[strum(to_string = "Dinosaur Comics")]
    DinosaurComics,
    #[strum(to_string = "Oglaf [NSFW]")]
    Oglaf,
    #[strum(to_string = "CTRL+ALT+DEL")]
    CadComics,
    #[strum(to_string = "The Joy of Tech")]
    JoyOfTech,
    #[strum(to_string = "Good Tech Things")]
    GoodTechThings,
    #[strum(to_string = "Three Word Phrase")]
    ThreeWordPhrase,
    #[strum(to_string = "a softer world")]
    ASofterWorld,
    #[strum(to_string = "BUTTERSAFE")]
    ButterSafe,
    #[strum(to_string = "Questionable Content")]
    QuestionableContent,
    #[strum(to_string = "Work Chronicles")]
    WorkChronicles,
    #[strum(to_string = "Junior Scientist Power Hour")]
    JSPowerHour,
    #[strum(to_string = "Buttercup Festival")]
    ButtercupFestival,
    #[strum(to_string = "achewood")]
    Achewood,
    #[strum(to_string = "Cat and Girl")]
    CatAndGirl,
    #[strum(to_string = "Diesel Sweeties #1.0")]
    DieselSweeties1_0,
    #[strum(to_string = "Diesel Sweeties #3.0")]
    DieselSweeties3_0,
    #[strum(to_string = "Poorly Drawn Lines")]
    PoorlyDrawnLines,
    #[strum(to_string = "Piled Higher and Deeper")]
    PiledHigherAndDeeper,
    #[strum(to_string = "Gunnerkrigg Court")]
    GunnerkriggCourt,
}

impl Sites {
    pub fn sites_sorted() -> Vec<Sites> {
        let mut sites: Vec<_> = Sites::iter().collect();
        sites.sort_by_key(|site| site.to_string().to_lowercase());
        sites
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

    pub fn is_last(&self) -> bool {
        self.strip_type == StripType::First || self.strip_type == StripType::Unique
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
            Sites::PoorlyDrawnLines => "https://poorlydrawnlines.com/feed",
            Sites::PiledHigherAndDeeper => "https://phdcomics.com/comics/archive_list.php",
            Sites::GunnerkriggCourt => "https://www.gunnerkrigg.com/archives",
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
            Sites::PoorlyDrawnLines => "poorlydrawnlines.com",
            Sites::PiledHigherAndDeeper => "phdcomics.com",
            Sites::GunnerkriggCourt => "gunnerkrigg.com",
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Fetcher, fetcher::build_fetcher};

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

    // Fails on gh ci
    #[ignore]
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

    #[tokio::test]
    async fn test_parse_poorly_drawn_lines_content() {
        let fetcher = build_fetcher(crate::Sites::PoorlyDrawnLines).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        println!("{:?}", fetcher.last().await);
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_parse_phd_content() {
        let fetcher = build_fetcher(crate::Sites::PiledHigherAndDeeper).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        println!("{:?}", fetcher.last().await);
        assert!(fetcher.random().await.is_ok());
    }

    #[tokio::test]
    async fn test_parse_gunnerkrigg_court_content() {
        let fetcher = build_fetcher(crate::Sites::GunnerkriggCourt).await;
        assert!(fetcher.is_some());
        let fetcher = fetcher.unwrap();
        assert!(fetcher.last().await.is_ok());
        println!("{:?}", fetcher.last().await);
        assert!(fetcher.random().await.is_ok());
    }
}
