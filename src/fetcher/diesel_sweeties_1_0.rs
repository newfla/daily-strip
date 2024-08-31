use anyhow::{bail, Result};

use crate::{FetcherErrors, Strip, StripType, Url};

use super::FetcherImpl;

impl FetcherImpl {
    pub(super) async fn reload_diesel_sweeties_1_0(&mut self) -> Result<()> {
        let mut data: Vec<_> = (1..=4000)
            .map(|idx| Strip {
                title: idx.to_string(),
                url: format!("{}/{idx}", self.site.fetch_url()),
                idx: idx - 1,
                strip_type: StripType::Unknown,
            })
            .collect();

        Self::reverse_strip_vec(&mut data);
        match data.len() {
            0 => bail!(FetcherErrors::Error404),
            _ => {
                self.posts = Some(data);
                Ok(())
            }
        }
    }

    pub(super) async fn parse_diesel_sweeties_1_0_content(&self, content: &Strip) -> Result<Strip> {
        //Something is wrong with site certificate
        let data = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?
            .get(&content.url)
            .send()
            .await?
            .text()
            .await?;
        let url = Self::parse_first_occurrence_blocking(
            &data,
            "center ~ table ~ div > table > tbody > tr td div > img",
            "src",
        )
        .ok_or(FetcherErrors::Error404)?;

        let title = Self::parse_first_occurrence_blocking(
            &data,
            "center ~ table ~ div > table > tbody > tr td div > img",
            "title",
        )
        .unwrap_or_else(|| content.title.clone());

        Ok(Strip {
            title,
            // Switch to http to avoid image not loading due to wrong certificates
            url: format!("http://{}{}", self.site.homepage(), url),
            idx: content.idx,
            strip_type: content.strip_type,
        })
    }
}
