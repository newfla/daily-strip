use anyhow::{Result, bail};
use rss::Channel;

use crate::{FetcherErrors, Strip, StripType, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_poorly_drawn_lines(&mut self) -> Result<()> {
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
                strip_type: StripType::Unknown,
                site: self.site,
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

    pub(super) async fn parse_poorly_drawn_lines_content(&self, content: &Strip) -> Result<Strip> {
        let data = reqwest::get(&content.url).await?.text().await?;
        let url = Self::parse_first_occurrence_blocking(&data, "figure.wp-block-image a", "href")
            .ok_or(FetcherErrors::Error404)?;

        Ok(Strip {
            title: content.title.to_string(),
            url,
            idx: content.idx,
            strip_type: content.strip_type,
            site: content.site,
        })
    }
}
