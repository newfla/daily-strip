use anyhow::{bail, Result};
use scraper::{Html, Selector};

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_softer_world(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("td a").unwrap();
        let mut data: Vec<_> = frag
            .select(&selector)
            .enumerate()
            .map(|(idx, elem)| {
                let url = elem.value().attr("href").unwrap().to_owned();
                Strip {
                    title: format!("https://{}", self.site.homepage()),
                    url,
                    idx,
                    strip_type: crate::StripType::Unknown,
                }
            })
            .skip(1)
            .collect();
        Self::reverse_strip_vec(&mut data);

        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    pub(super) async fn parse_softer_world_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(&data, "#comicimg img", "src")
            .ok_or(FetcherErrors::Error404)?
            .replace("..", &content.title);

        let title = Self::parse_first_occurency_blocking(&data, "#comicimg img", "title")
            .ok_or(FetcherErrors::Error404)?;
        Ok(Strip {
            title,
            url,
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
