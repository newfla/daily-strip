use anyhow::{bail, Result};
use rss::Channel;

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_cmd(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.bytes().await?;
        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.link))
            .filter(|(title, link)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && link
                        .as_ref()
                        .is_some_and(|description| !description.is_empty())
            })
            .enumerate()
            .map(|(idx, (title, url))| Strip {
                title: title.unwrap(),
                url: url.unwrap(),
                idx,
                strip_type: crate::StripType::Unknown,
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

    pub(super) async fn parse_cad_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurency_blocking(&data, "div.arrowright + a img", "src")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url,
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
