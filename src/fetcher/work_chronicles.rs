use anyhow::{bail, Result};
use scraper::{Html, Selector};

use super::FetcherImpl;
use crate::FetcherErrors::Error404;
use crate::{FetcherErrors, Strip, StripType, Url};

impl FetcherImpl {
    pub(super) async fn reload_work_chronicles(&mut self) -> Result<()> {
        let mut data = Vec::new();
        let mut counter = 0;

        let selector = Selector::parse("a.sitemap-link").map_err(|_| Error404)?;

        let urls: Vec<_> = {
            let data = reqwest::get(format!("{}/sitemap", self.site.fetch_url()))
                .await?
                .text()
                .await?;
            let frag = Html::parse_document(&data);
            frag.select(&selector)
                .map(|elem| elem.attr("href").unwrap().to_owned())
                .collect()
        };

        for url in urls.iter() {
            let html = reqwest::get(format!("{}{url}", self.site.fetch_url()))
                .await?
                .text()
                .await?;
            let frag = Html::parse_document(&html);
            for elem in frag.select(&selector) {
                let inner_html = elem.inner_html();
                let title_split = inner_html.split_once("(comic) ");

                if let Some((_, title)) = title_split {
                    let url = elem.value().attr("href").unwrap().to_owned();
                    data.push(Strip {
                        title: title.to_owned(),
                        url,
                        idx: counter,
                        strip_type: StripType::Unknown,
                    });
                    counter += 1;
                }
            }
        }

        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    pub(super) async fn parse_work_chronicles_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "figure a.image-link", "href")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.clone(),
            url,
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
