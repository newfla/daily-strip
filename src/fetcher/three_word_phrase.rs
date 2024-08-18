use anyhow::{bail, Result};
use scraper::{Html, Selector};

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_three_word_phrase(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let frag = Html::parse_document(&data);
        let selector = Selector::parse("span.links a").unwrap();
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
                url: format!(
                    "https://{}/{}",
                    self.site.homepage(),
                    url.unwrap().to_owned()
                ),
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

    pub(super) async fn parse_three_word_phrase_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(&data, "td center img", "src")
            .ok_or(FetcherErrors::Error404)?
            .replace("..", &content.title);
        Ok(Strip {
            title: content.title.clone(),
            url: format!("https://{}/{}", self.site.homepage(), url),
        })
    }
}
