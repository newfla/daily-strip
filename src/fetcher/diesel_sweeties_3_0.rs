use anyhow::{bail, Result};
use rss::Channel;

use crate::{FetcherErrors, Strip, StripType, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_diesel_sweeties_3_0(&mut self) -> Result<()> {
        //Something is wrong with site certificate
        let data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?
            .get(self.site.fetch_url())
            .send()
            .await?
            .bytes()
            .await?;

        let data: Vec<_> = Channel::read_from(&data[..])?
            .items
            .into_iter()
            .map(|item| (item.title, item.description))
            .filter(|(title, description)| {
                title.as_ref().is_some_and(|title| !title.is_empty())
                    && description
                        .as_ref()
                        .is_some_and(|description| !description.is_empty())
            })
            .enumerate()
            .map(|(idx, (title, description))| {
                let url =
                    Self::parse_first_occurrence_blocking(&description.unwrap(), "img", "src");
                Strip {
                    title: title.unwrap(),
                    url: url.unwrap(),
                    idx,
                    strip_type: StripType::Unknown,
                    site: self.site,
                }
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

    pub(super) async fn parse_diesel_sweeties_3_0_content(&self, content: &Strip) -> Result<Strip> {
        Ok(content.clone())
    }
}
