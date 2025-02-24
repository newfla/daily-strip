use anyhow::{Result, bail};
use scraper::{Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_gunnerkrigg_court(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("option").map_err(|_| Error404)?;
        let limit = frag
            .select(&selector)
            .last()
            .map(|elem| elem.attr("value").unwrap().parse::<usize>().unwrap())
            .unwrap();
        let mut data: Vec<_> = (1..=limit)
            .map(|idx| Strip {
                title: idx.to_string(),
                url: format!("https://{}/comics/{:08}.jpg", self.site.homepage(), idx),
                idx: idx - 1,
                strip_type: StripType::Unknown,
                site: self.site,
            })
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

    pub(super) async fn parse_gunnerkrigg_court_content(&self, content: &Strip) -> Result<Strip> {
        Ok(Strip {
            title: content.title.clone(),
            url: content.url.clone(),
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
