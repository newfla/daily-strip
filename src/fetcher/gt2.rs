use anyhow::{bail, Result};
use rss::Channel;

use crate::{FetcherErrors, Strip, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_gt2(&mut self) -> Result<()> {
        let data = reqwest::get(self.site.fetch_url()).await?.bytes().await?;
        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.content))
            .filter(|(title, content)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && content
                        .as_ref()
                        .is_some_and(|description| !description.is_empty())
            })
            .enumerate()
            .map(|(idx, (title, content))| Strip {
                title: title.unwrap(),
                url: content.unwrap(),
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

    pub(super) async fn parse_gt2_content(&self, content: &Strip) -> Result<Strip> {
        let url = Self::parse_first_occurency_blocking(&content.url, "img", "src")
            .ok_or(FetcherErrors::Error404)?
            .replace("..", &content.title);
        Ok(Strip {
            title: content.title.clone(),
            url,
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
