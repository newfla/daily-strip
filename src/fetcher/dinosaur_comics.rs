use anyhow::{bail, Result};
use scraper::{Element, Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_dinosaur_comics(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url().to_owned() + "/archive.php")
            .await?
            .text()
            .await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("ul.archive li a").map_err(|_| Error404)?;
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
            .enumerate()
            .map(|(idx, (title, url))| Strip {
                title,
                url: url.unwrap().to_string(),
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

    pub(super) async fn parse_dinosaur_comics_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "img.comic", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.clone(),
            url: self.site.fetch_url().to_owned() + "/" + &url,
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
