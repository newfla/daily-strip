mod achewood;
mod butter_safe;
mod buttercup_festival;
mod cad_comics;
mod cat_and_girl;
mod cornet_comics;
mod diesel_sweeties_1_0;
mod diesel_sweeties_3_0;
mod dinosaur_comics;
mod gt2;
mod gunnerkrigg_court;
mod joy_of_tech;
mod js_power_hour;
mod monkey_user;
mod oglaf;
mod phd;
mod poorly_drawn_lines;
mod questionable_content;
mod softer_world;
mod three_word_phrase;
mod turnoff_us;
mod work_chronicles;
mod xkcd;

use anyhow::{bail, Result};
use async_trait::async_trait;
use rand::{rng, Rng};
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
            Sites::QuestionableContent => self.reload_questionable_content().await,
            Sites::WorkChronicles => self.reload_work_chronicles().await,
            Sites::JSPowerHour => self.reload_js_power_hour().await,
            Sites::ButtercupFestival => self.reload_buttercup_festival().await,
            Sites::Achewood => self.reload_achewood().await,
            Sites::CatAndGirl => self.reload_cat_and_girl().await,
            Sites::DieselSweeties1_0 => self.reload_diesel_sweeties_1_0().await,
            Sites::DieselSweeties3_0 => self.reload_diesel_sweeties_3_0().await,
            Sites::PoorlyDrawnLines => self.reload_poorly_drawn_lines().await,
            Sites::PiledHigherAndDeeper => self.reload_phd().await,
            Sites::GunnerkriggCourt => self.reload_gunnerkrigg_court().await,
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
        let mut random = rng();
        self.posts
            .as_ref()
            .and_then(|data| data.get(random.random_range(0..data.len())))
    }

    fn idx_content(&self, idx: usize) -> Option<&Strip> {
        self.posts.as_ref().and_then(|data| data.get(idx))
    }

    async fn parse_content(&self, content: &Strip) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.parse_turnoff_us_content(content).await,
            Sites::MonkeyUser => self.parse_monkey_user_content(content).await,
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
            Sites::QuestionableContent => self.parse_questionable_content_content(content).await,
            Sites::WorkChronicles => self.parse_work_chronicles_content(content).await,
            Sites::JSPowerHour => self.parse_js_power_hour_content(content).await,
            Sites::ButtercupFestival => self.parse_buttercup_festival_content(content).await,
            Sites::Achewood => self.parse_achewood_content(content).await,
            Sites::CatAndGirl => self.parse_cat_and_girl_content(content).await,
            Sites::DieselSweeties1_0 => self.parse_diesel_sweeties_1_0_content(content).await,
            Sites::DieselSweeties3_0 => self.parse_diesel_sweeties_3_0_content(content).await,
            Sites::PoorlyDrawnLines => self.parse_poorly_drawn_lines_content(content).await,
            Sites::PiledHigherAndDeeper => self.parse_phd_content(content).await,
            Sites::GunnerkriggCourt => self.parse_gunnerkrigg_court_content(content).await,
        }
    }

    fn parse_first_occurrence_blocking(data: &str, selector: &str, attr: &str) -> Option<String> {
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

    fn reverse_strip_vec(data: &mut [Strip]) {
        data.reverse();
        data.iter_mut()
            .enumerate()
            .for_each(|(idx, strip)| strip.idx = idx);
    }
}
