use anyhow::{bail, Result};
use scraper::{Html, Selector};

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_work_chronicles(&mut self) -> Result<()> {
        let mut data = Vec::new();
        let mut counter = 0;

        let urls: Vec<_> = {
            let data = reqwest::get(format!("{}/sitemap", self.site.fetch_url()))
                .await?
                .text()
                .await?;
            let frag = Html::parse_document(&data);
            let selector = Selector::parse("a.sitemap-link").unwrap();
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
            let selector = Selector::parse("a.sitemap-link").unwrap();
            for elem in frag.select(&selector) {
                let inner_html = elem.inner_html();
                let title_splitted = inner_html.split_once("(comic) ");

                if let Some((_, title)) = title_splitted {
                    let url = elem.value().attr("href").unwrap().to_owned();
                    data.push(Strip {
                        title: title.to_owned(),
                        url,
                        idx: counter,
                        strip_type: crate::StripType::Unknown,
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
        let url = Self::parse_first_occurency_blocking(&data, "figure a.image-link", "href")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.clone(),
            url,
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
