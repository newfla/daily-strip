use anyhow::{bail, Result};
use async_trait::async_trait;
use rand::{thread_rng, Rng};
use rss::Channel;
use scraper::{Element, Html, Selector};

use crate::{Fetcher, FetcherErrors, Sites, Strip};

struct Content {
    title: String,
    url: String,
}
struct FetcherImpl {
    site: Sites,
    posts: Option<Vec<Content>>,
}

#[async_trait]
impl Fetcher for FetcherImpl {
    async fn reload(&mut self) -> Result<()> {
        match self.site {
            Sites::TurnoffUs => self.reload_turnoff_us().await,
            Sites::MonkeyUser => self.reload_monkey_user().await,
            Sites::BonkersWorld => self.reload_cornet_comics().await,
            Sites::Goomics => self.reload_cornet_comics().await,
            Sites::Xkcd => self.reload_xkcd().await,
            Sites::Oglaf => self.reload_oglaf().await,
            Sites::DinosaurComics => self.reload_dinosaur_comics().await,
        }
    }

    async fn last(&mut self) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.last_turnoff_us().await,
            Sites::MonkeyUser => self.last_generic_strip().await,
            Sites::BonkersWorld => self.last_generic_strip().await,
            Sites::Goomics => self.last_generic_strip().await,
            Sites::Xkcd => self.last_xkcd().await,
            Sites::Oglaf => self.last_oglaf().await,
            Sites::DinosaurComics => self.last_dinosaur_comics().await,
        }
    }

    async fn random(&mut self) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.random_turnoff_us().await,
            Sites::MonkeyUser => self.random_generic_strip().await,
            Sites::BonkersWorld => self.random_generic_strip().await,
            Sites::Goomics => self.random_generic_strip().await,
            Sites::Xkcd => self.random_xkcd().await,
            Sites::Oglaf => self.random_oglaf().await,
            Sites::DinosaurComics => self.random_dinosaur_comics().await,
        }
    }
}

pub async fn build_fetcher(site: Sites) -> Option<impl Fetcher> {
    let posts = None;
    let mut fetcher = FetcherImpl { site, posts };
    fetcher.reload().await.ok().map(|_| fetcher)
}

impl FetcherImpl {
    fn random_elem(&self) -> Option<&Content> {
        let mut random = thread_rng();
        self.posts
            .as_ref()
            .and_then(|data| data.get(random.gen_range(0..data.len())))
    }

    async fn reload_turnoff_us(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string() + "/all")
            .await?
            .text()
            .await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("a.post-link").unwrap();
        let data: Vec<_> = frag
            .select(&selector)
            .map(|element| {
                (
                    element.inner_html().trim().to_string(),
                    element.value().attr("href"),
                )
            })
            .filter(|(title, url)| !title.is_empty() && url.is_some())
            .map(|(title, url)| Content {
                title,
                url: self.site.to_string() + url.unwrap(),
            })
            .collect();

        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    async fn reload_monkey_user(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string()).await?.bytes().await?;
        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.description))
            .filter(|(title, description)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && description
                        .as_ref()
                        .is_some_and(|description| !description.is_empty())
            })
            .map(|(name, description)| {
                (
                    name.unwrap(),
                    Self::parse_first_occurency_blocking(description.unwrap(), "p img", "src"),
                )
            })
            .filter(|(_, url)| url.is_some())
            .map(|(name, url)| Content {
                title: name,
                url: url.unwrap(),
            })
            .collect();
        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    async fn reload_oglaf(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string()).await?.bytes().await?;
        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.description))
            .filter(|(title, description)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && description
                        .as_ref()
                        .is_some_and(|description| !description.is_empty())
            })
            .map(|(name, description)| {
                (
                    name.unwrap(),
                    Self::parse_first_occurency_blocking(description.unwrap(), "p a", "href"),
                )
            })
            .filter(|(_, url)| url.is_some())
            .map(|(name, url)| Content {
                title: name,
                url: url.unwrap(),
            })
            .collect();
        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    async fn reload_dinosaur_comics(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string() + "/archive.php")
            .await?
            .text()
            .await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("ul.archive li a").unwrap();
        let data: Vec<_> = frag
            .select(&selector)
            .map(|element| {
                (
                    element
                        .parent_element()
                        .unwrap()
                        .text()
                        .map(|s| s.to_string())
                        .reduce(|cur, nxt| cur + &nxt)
                        .unwrap_or("".to_string()),
                    element.value().attr("href"),
                )
            })
            .filter(|(title, url)| !title.is_empty() && url.is_some())
            .map(|(title, url)| Content {
                title,
                url: url.unwrap().to_string(),
            })
            .collect();

        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    async fn reload_cornet_comics(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector_name = Selector::parse("span a.post-link").unwrap();
        let selector_url = Selector::parse("a.post-link img").unwrap();

        let data: Vec<_> = frag
            .select(&selector_name)
            .zip(frag.select(&selector_url))
            .map(|(a_title, img_url)| (a_title.inner_html(), img_url.value().attr("src")))
            .filter(|(title, thumb_url)| !title.is_empty() && thumb_url.is_some())
            .map(|(name, thumb_url)| Content {
                title: name.trim().to_string(),
                url: self.site.to_string()
                    + &thumb_url
                        .unwrap()
                        .to_string()
                        .replace("thumbs/", "")
                        .replace("_thumbnail", ""),
            })
            .collect();
        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    async fn reload_xkcd(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.to_string()).await?.text().await?;
        let last = self
            .parse_meta_content_blocking(data, "og:url")
            .ok_or(FetcherErrors::Error404)?
            .replace('/', "");
        let mut data = Vec::new();
        for i in (1..(1 + last.parse::<usize>()?)).rev() {
            data.push(Content {
                title: i.to_string(),
                url: self.site.to_string() + "/" + &i.to_string(),
            })
        }
        self.posts = Some(data);
        Ok(())
    }

    async fn last_turnoff_us(&mut self) -> Result<Strip> {
        match self.posts.as_ref() {
            Some(data) => match data.get(0) {
                Some(content) => self.parse_turnoff_us_content(content).await,
                None => bail!(FetcherErrors::Error404),
            },
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_generic_strip(&mut self) -> Result<Strip> {
        match self.posts.as_ref() {
            Some(data) => match data.get(0) {
                Some(content) => Self::parse_generic_content(content).await,
                None => bail!(FetcherErrors::Error404),
            },
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_xkcd(&mut self) -> Result<Strip> {
        match self.posts.as_ref() {
            Some(data) => match data.get(0) {
                Some(content) => self.parse_xkcd_content(content).await,
                None => bail!(FetcherErrors::Error404),
            },
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_oglaf(&mut self) -> Result<Strip> {
        match self.posts.as_ref() {
            Some(data) => match data.get(0) {
                Some(content) => self.parse_oglaf_content(content).await,
                None => bail!(FetcherErrors::Error404),
            },
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_dinosaur_comics(&mut self) -> Result<Strip> {
        match self.posts.as_ref() {
            Some(data) => match data.get(0) {
                Some(content) => self.parse_dinosaur_comics_content(content).await,
                None => bail!(FetcherErrors::Error404),
            },
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_turnoff_us(&mut self) -> Result<Strip> {
        match self.random_elem() {
            Some(content) => self.parse_turnoff_us_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_generic_strip(&mut self) -> Result<Strip> {
        match self.random_elem() {
            Some(content) => Self::parse_generic_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_xkcd(&mut self) -> Result<Strip> {
        match self.random_elem() {
            Some(content) => self.parse_xkcd_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_oglaf(&mut self) -> Result<Strip> {
        match self.random_elem() {
            Some(content) => self.parse_oglaf_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_dinosaur_comics(&mut self) -> Result<Strip> {
        match self.random_elem() {
            Some(content) => self.parse_dinosaur_comics_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn parse_turnoff_us_content(&self, content: &Content) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "p img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Self::parse_generic_content(&Content {
            title: content.title.to_string(),
            url: self.site.to_string() + &url,
        })
        .await
    }

    async fn parse_xkcd_content(&self, content: &Content) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = self
            .parse_meta_content_blocking(data, "og:image")
            .ok_or(FetcherErrors::Error404)?;

        Self::parse_generic_content(&Content {
            title: content.title.to_string(),
            url,
        })
        .await
    }

    async fn parse_oglaf_content(&self, content: &Content) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "#strip", "src")
            .ok_or(FetcherErrors::Error404)?;

        Self::parse_generic_content(&Content {
            title: content.title.to_string(),
            url: self.site.to_string() + &url,
        })
        .await
    }

    async fn parse_dinosaur_comics_content(&self, content: &Content) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "img.comic", "src")
            .ok_or(FetcherErrors::Error404)?;

        Self::parse_generic_content(&Content {
            title: content.title.to_string(),
            url: self.site.to_string() + "/" + &url,
        })
        .await
    }

    async fn parse_generic_content(content: &Content) -> Result<Strip> {
        Ok((
            content.title.clone(),
            reqwest::get(&content.url).await?.bytes().await?.to_vec(),
        ))
    }

    fn parse_first_occurency_blocking(data: String, selector: &str, attr: &str) -> Option<String> {
        let frag = Html::parse_document(&data);
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
                    .replace(&self.site.to_string(), "")
            })
            .last()
    }
}
