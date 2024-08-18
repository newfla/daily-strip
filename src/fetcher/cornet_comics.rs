use anyhow::{bail, Result};
use scraper::{Html, Selector};

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_cornet_comics(&mut self) -> Result<()> {
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
                url: self.site.fetch_url().to_owned()
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

    pub(super) async fn parse_cornet_content(&self, content: &Strip) -> Result<Strip> {
        Ok(content.clone())
    }
}
