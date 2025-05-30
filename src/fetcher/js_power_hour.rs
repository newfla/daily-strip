use anyhow::{Result, bail};
use scraper::{Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_js_power_hour(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("div.archive-comic a").map_err(|_| Error404)?;
        let data: Vec<_> = frag
            .select(&selector)
            .enumerate()
            .map(|(idx, elem)| {
                let url = elem.value().attr("href").unwrap().to_owned();
                let title = url.split_once("/comics/").unwrap().1.to_owned();
                Strip {
                    title,
                    url: format!("https://{}{}", self.site.homepage(), url),
                    idx,
                    strip_type: StripType::Unknown,
                    site: self.site,
                }
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

    pub(super) async fn parse_js_power_hour_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "#comic-img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.clone(),
            url: format!("https:{url}"),
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
