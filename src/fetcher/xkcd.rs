use anyhow::Result;

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_xkcd(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.text().await?;
        let last = self
            .parse_meta_content_blocking(data, "og:url")
            .ok_or(FetcherErrors::Error404)?
            .replace('/', "");
        let mut data = Vec::new();
        for i in (1..(1 + last.parse::<usize>()?)).rev() {
            data.push(Strip {
                title: i.to_string(),
                url: self.site.fetch_url().to_owned() + "/" + &i.to_string(),
            })
        }
        self.posts = Some(data);
        Ok(())
    }

    pub(super) async fn parse_xkcd_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = self
            .parse_meta_content_blocking(data, "og:image")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url,
        })
    }
}
