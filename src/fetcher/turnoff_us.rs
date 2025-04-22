use anyhow::{Result, bail};
use scraper::{Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_turnoff_us(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url().to_owned() + "/all")
            .await?
            .text()
            .await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("a.post-link").map_err(|_| Error404)?;
        let data: Vec<_> = frag
            .select(&selector)
            .map(|element| {
                (
                    element.inner_html().trim().to_string(),
                    element.value().attr("href"),
                )
            })
            .filter(|(title, url)| !title.is_empty() && url.is_some())
            .enumerate()
            .map(|(idx, (title, url))| Strip {
                title,
                url: self.site.fetch_url().to_owned() + url.unwrap(),
                idx,
                strip_type: StripType::Unknown,
                site: self.site,
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

    pub(super) async fn parse_turnoff_us_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "p img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url: self.site.fetch_url().to_owned() + &url,
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
