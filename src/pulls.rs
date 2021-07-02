use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequest {
    pub url: String,
    pub id: u64,
    pub number: u64,
    pub node_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequestFile {
    pub sha: String,
    pub filename: String,
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub blob_url: Url,
    pub raw_url: Url,
    pub contents_url: Url,
    pub patch: String,

    pub pull_request_number: Option<u64>,
    pub sdc_repository: Option<String>,
}

pub struct PullFileFetcher {
    owner: String,
    name: String,
    since: Option<DateTime<Utc>>,
    octocrab: octocrab::Octocrab,
}

impl PullFileFetcher {
    pub fn new(
        owner: String,
        name: String,
        since: Option<DateTime<Utc>>,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            since,
            octocrab,
        }
    }
}

impl PullFileFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let param = Params::default();
        let route = format!(
            "repos/{owner}/{repo}/pulls?{query}&state=all&sort=updated",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        let mut next: Option<Url> = self.octocrab.absolute_url(route).ok();

        while let Some(mut page) = self.octocrab.get_page(&next).await? {
            let pulls: Vec<PullRequest> = page.take_items();
            let mut last_update: Option<DateTime<Utc>> = None;
            for pull in pulls.into_iter() {
                let route = format!(
                    "repos/{owner}/{repo}/pulls/{number}/files",
                    owner = &self.owner,
                    repo = &self.name,
                    number = pull.number,
                );
                let files_url: Url = self.octocrab.absolute_url(route).unwrap();
                let mut files: Vec<PullRequestFile> =
                    self.octocrab.get(&files_url, None::<&()>).await?;
                for file in files.iter_mut() {
                    file.pull_request_number = pull.number.into();
                    file.sdc_repository = format!("{}/{}", self.owner, self.name).into();

                    wtr.serialize(file).expect("Serialize failed");
                }

                last_update = Some(pull.updated_at.unwrap_or_else(|| pull.created_at));
            }

            next = if let Some(since) = self.since {
                if last_update.unwrap() < since {
                    None
                } else {
                    page.next
                }
            } else {
                page.next
            };
        }

        Ok(())
    }
}
