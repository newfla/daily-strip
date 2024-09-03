use anyhow::{bail, Result};
use scraper::{Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_buttercup_festival(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("a").map_err(|_| Error404)?;
        let mut data: Vec<_> = frag
            .select(&selector)
            .filter_map(|elem| {
                elem.inner_html()
                    .parse::<usize>()
                    .map(|idx| {
                        let url = elem.value().attr("href").unwrap();
                        Strip {
                            title: url.split_once('.').unwrap().0.to_owned(),
                            url: format!("{}/{}", self.site.fetch_url(), url),
                            idx,
                            strip_type: StripType::Unknown,
                            site: self.site,
                        }
                    })
                    .ok()
            })
            .collect();

        data.sort_by_key(|s| format!("{}-{:03}", s.title.split_once('-').unwrap().0, s.idx));
        Self::reverse_strip_vec(&mut data);
        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    pub(super) async fn parse_buttercup_festival_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "center img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.clone(),
            url: format!("{}/{}", self.site.fetch_url(), url),
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
