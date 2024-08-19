mod butter_safe;
mod cad_comics;
mod cornet_comics;
mod dinosaur_comics;
mod gt2;
mod joy_of_tech;
mod monkey_user;
mod oglaf;
mod softer_world;
mod three_word_phrase;
mod turnoff_us;
mod xkcd;

use anyhow::{bail, Result};
use async_trait::async_trait;
use rand::{thread_rng, Rng};
use scraper::{Html, Selector};

use crate::{Fetcher, FetcherErrors, Sites, Strip, StripType, Url};

struct FetcherImpl {
    site: Sites,
    posts: Option<Vec<Strip>>,
}

#[async_trait]
impl Fetcher for FetcherImpl {
    async fn reload(&mut self) -> Result<()> {
        let res = match self.site {
            Sites::TurnoffUs => self.reload_turnoff_us().await,
            Sites::MonkeyUser => self.reload_monkey_user().await,
            Sites::BonkersWorld => self.reload_cornet_comics().await,
            Sites::Goomics => self.reload_cornet_comics().await,
            Sites::Xkcd => self.reload_xkcd().await,
            Sites::Oglaf => self.reload_oglaf().await,
            Sites::DinosaurComics => self.reload_dinosaur_comics().await,
            Sites::CadComics => self.reload_cmd().await,
            Sites::JoyOfTech => self.reload_joy_of_tech().await,
            Sites::GoodTechThings => self.reload_gt2().await,
            Sites::ThreeWordPhrase => self.reload_three_word_phrase().await,
            Sites::ASofterWorld => self.reload_softer_world().await,
            Sites::ButterSafe => self.reload_butter_safe().await,
        };
        self.set_strip_type();
        res
    }

    async fn last(&self) -> Result<Strip> {
        match self.last_content() {
            Some(content) => self.parse_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random(&self) -> Result<Strip> {
        match self.random_content() {
            Some(content) => self.parse_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn next(&self, idx: usize) -> Result<Strip> {
        if idx == 0 {
            bail!(FetcherErrors::Error404)
        }

        match self.idx_content(idx - 1) {
            Some(content) => self.parse_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn prev(&self, idx: usize) -> Result<Strip> {
        match self.idx_content(idx + 1) {
            Some(content) => self.parse_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }
}

pub async fn build_fetcher(site: Sites) -> Option<impl Fetcher> {
    let posts = None;
    let mut fetcher = FetcherImpl { site, posts };
    fetcher.reload().await.ok().map(|_| fetcher)
}

impl FetcherImpl {
    fn set_strip_type(&mut self) {
        if let Some(data) = self.posts.as_deref_mut() {
            if let Some(elem) = data.first_mut() {
                elem.strip_type = StripType::First;
            }
            if let Some(elem) = data.last_mut() {
                elem.strip_type = StripType::Last;
            }

            if data.len() == 1 {
                data.first_mut().unwrap().strip_type = StripType::Unique
            }
        }
    }

    fn last_content(&self) -> Option<&Strip> {
        match self.posts.as_ref() {
            Some(data) => data.first(),
            None => None,
        }
    }

    fn random_content(&self) -> Option<&Strip> {
        let mut random = thread_rng();
        self.posts
            .as_ref()
            .and_then(|data| data.get(random.gen_range(0..data.len())))
    }

    fn idx_content(&self, idx: usize) -> Option<&Strip> {
        self.posts.as_ref().and_then(|data| data.get(idx))
    }

    async fn parse_content(&self, content: &Strip) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.parse_turnoff_us_content(content).await,
            Sites::MonkeyUser => self.parse_monkeyuser_content(content).await,
            Sites::BonkersWorld => self.parse_cornet_content(content).await,
            Sites::Goomics => self.parse_cornet_content(content).await,
            Sites::Xkcd => self.parse_xkcd_content(content).await,
            Sites::Oglaf => self.parse_oglaf_content(content).await,
            Sites::DinosaurComics => self.parse_dinosaur_comics_content(content).await,
            Sites::CadComics => self.parse_cad_content(content).await,
            Sites::JoyOfTech => self.parse_joy_of_tech_content(content).await,
            Sites::GoodTechThings => self.parse_gt2_content(content).await,
            Sites::ThreeWordPhrase => self.parse_three_word_phrase_content(content).await,
            Sites::ASofterWorld => self.parse_softer_world_content(content).await,
            Sites::ButterSafe => self.parse_butter_safe_content(content).await,
        }
    }

    fn parse_first_occurency_blocking(data: &str, selector: &str, attr: &str) -> Option<String> {
        let frag = Html::parse_document(data);
        let selector = Selector::parse(selector).unwrap();
        Some(
            frag.select(&selector)
                .next()?
                .value()
                .attr(attr)?
                .to_string(),
        )
    }

    fn parse_meta_content_blocking(&self, data: String, property: &str) -> Option<String> {
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("meta").unwrap();
        frag.select(&selector)
            .filter(|elem| elem.value().attr("property") == Some(property))
            .map(|elem| {
                elem.attr("content")
                    .unwrap()
                    .replace(self.site.fetch_url(), "")
            })
            .last()
    }
}
