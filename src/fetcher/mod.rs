use anyhow::{bail, Ok, Result};
use async_trait::async_trait;
use rand::{thread_rng, Rng};
use rss::Channel;
use scraper::{Element, Html, Selector};

use crate::{Fetcher, FetcherErrors, Sites, Strip, Url};

struct FetcherImpl {
    site: Sites,
    posts: Option<Vec<Strip>>,
}

#[async_trait]
impl Fetcher for FetcherImpl {
    async fn reload(&mut self) -> Result<()> {
        match self.site {
            Sites::TurnoffUs => self.reload_turnoff_us().await,
            Sites::MonkeyUser => self.reload_monkeyuser().await,
            Sites::BonkersWorld => self.reload_cornet_comics().await,
            Sites::Goomics => self.reload_cornet_comics().await,
            Sites::Xkcd => self.reload_xkcd().await,
            Sites::Oglaf => self.reload_oglaf().await,
            Sites::DinosaurComics => self.reload_dinosaur_comics().await,
        }
    }

    async fn last(&self) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.last_turnoff_us().await,
            Sites::MonkeyUser => self.last_monkeyuser().await,
            Sites::BonkersWorld => self.last_generic_strip().await,
            Sites::Goomics => self.last_generic_strip().await,
            Sites::Xkcd => self.last_xkcd().await,
            Sites::Oglaf => self.last_oglaf().await,
            Sites::DinosaurComics => self.last_dinosaur_comics().await,
        }
    }

    async fn random(&self) -> Result<Strip> {
        match self.site {
            Sites::TurnoffUs => self.random_turnoff_us().await,
            Sites::MonkeyUser => self.random_monkeyuser().await,
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
    fn last_content(&self) -> Option<Strip> {
        match self.posts.as_ref() {
            Some(data) => data.first().cloned(),
            None => None,
        }
    }

    fn random_content(&self) -> Option<Strip> {
        let mut random = thread_rng();
        self.posts
            .as_ref()
            .and_then(|data| data.get(random.gen_range(0..data.len())))
            .cloned()
    }

    async fn reload_turnoff_us(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url() + "/all")
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
            .map(|(title, url)| Strip {
                title,
                url: self.site.fetch_url() + url.unwrap(),
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

    async fn reload_monkeyuser(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.bytes().await?;
        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.link))
            .filter(|(title, link)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && link.as_ref().is_some_and(|link| !link.is_empty())
            })
            .map(|(name, link)| Strip {
                title: name.unwrap(),
                url: link.unwrap(),
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
        let data = reqwest::get(self.site.fetch_url()).await?.bytes().await?;
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
            .map(|(name, url)| Strip {
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
        let data = reqwest::get(self.site.fetch_url() + "/archive.php")
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
            .map(|(title, url)| Strip {
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
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector_name = Selector::parse("span a.post-link").unwrap();
        let selector_url = Selector::parse("a.post-link img").unwrap();

        let data: Vec<_> = frag
            .select(&selector_name)
            .zip(frag.select(&selector_url))
            .map(|(a_title, img_url)| (a_title.inner_html(), img_url.value().attr("src")))
            .filter(|(title, thumb_url)| !title.is_empty() && thumb_url.is_some())
            .map(|(name, thumb_url)| Strip {
                title: name.trim().to_string(),
                url: self.site.fetch_url()
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
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let last = self
            .parse_meta_content_blocking(data, "og:url")
            .ok_or(FetcherErrors::Error404)?
            .replace('/', "");
        let mut data = Vec::new();
        for i in (1..(1 + last.parse::<usize>()?)).rev() {
            data.push(Strip {
                title: i.to_string(),
                url: self.site.fetch_url() + "/" + &i.to_string(),
            })
        }
        self.posts = Some(data);
        Ok(())
    }

    async fn last_turnoff_us(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => self.parse_turnoff_us_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_monkeyuser(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => self.parse_monkeyuser_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_generic_strip(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => Ok(content.clone()),
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_xkcd(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => self.parse_xkcd_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_oglaf(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => self.parse_oglaf_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn last_dinosaur_comics(&self) -> Result<Strip> {
        match self.last_content().as_ref() {
            Some(content) => self.parse_dinosaur_comics_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_turnoff_us(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => self.parse_turnoff_us_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_generic_strip(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => Ok(content.clone()),
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_monkeyuser(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => self.parse_monkeyuser_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_xkcd(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => self.parse_xkcd_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_oglaf(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => self.parse_oglaf_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn random_dinosaur_comics(&self) -> Result<Strip> {
        match self.random_content().as_ref() {
            Some(content) => self.parse_dinosaur_comics_content(content).await,
            None => bail!(FetcherErrors::Error404),
        }
    }

    async fn parse_turnoff_us_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "p img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url: self.site.fetch_url() + &url,
        })
    }

    async fn parse_monkeyuser_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "p img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url: "https://".to_string() + &self.site.homepage() +&url,
        })
    }

    async fn parse_xkcd_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = self
            .parse_meta_content_blocking(data, "og:image")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url,
        })
    }

    async fn parse_oglaf_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "#strip", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url,
        })
    }

    async fn parse_dinosaur_comics_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(data, "img.comic", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url: self.site.fetch_url() + "/" + &url,
        })
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
                    .replace(&self.site.fetch_url(), "")
            })
            .last()
    }
}
