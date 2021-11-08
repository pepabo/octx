use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::*;

//use crate::commits::{Commit, GitCommit, GitUser, Object, UserId};
use crate::commits::Commit;
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
    pub sha: Option<String>,
    pub filename: Option<String>,
    pub additions: i32,
    pub deletions: i32,
    pub changes: i32,
    pub blob_url: Option<Url>,
    pub raw_url: Option<Url>,
    pub contents_url: Option<Url>,
    pub patch: Option<String>,

    pub pull_request_number: Option<u64>,
    pub sdc_repository: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct PrCommitRec {
    pub sha: Option<String>,
    pub node_id: Option<String>,
    pub url: Option<String>,
    pub html_url: Option<String>,
    pub comments_url: Option<String>,
    pub author_id: Option<i64>,
    pub committer_id: Option<i64>,
    pub author: Option<String>,    // Vec.to_json
    pub committer: Option<String>, // Vec.to_json
    pub parents: String,           // Vec.to_json
    pub message: Option<String>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub committed_at: Option<DateTime<Utc>>,
    pub comment_count: i32,

    pub pull_request_number: Option<u64>,

    pub sdc_repository: String,
}

impl RepositryAware for PrCommitRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<Commit> for PrCommitRec {
    fn from(from: Commit) -> Self {
        Self {
            sha: from.sha,
            node_id: from.node_id,
            url: from.url,
            html_url: from.html_url,
            comments_url: from.comments_url,
            author_id: from.author.map(|u| u.id).flatten(),
            committer_id: from.committer.map(|u| u.id).flatten(),
            author: from
                .commit
                .author
                .as_ref()
                .map(|d| serde_json::to_string(d).ok())
                .flatten(),
            committer: from
                .commit
                .committer
                .as_ref()
                .map(|d| serde_json::to_string(d).ok())
                .flatten(),
            parents: from
                .parents
                .iter()
                .map(|v| v.sha.to_owned())
                .collect::<Vec<String>>()
                .join(" "),
            message: from.commit.message,
            authorized_at: from.commit.author.as_ref().map(|a| a.date),
            committed_at: from.commit.committer.as_ref().map(|a| a.date),
            comment_count: from.commit.comment_count,

            pull_request_number: None,

            sdc_repository: String::default(),
        }
    }
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
            "repos/{owner}/{repo}/pulls?{query}&state=all&sort=updated&direction=desc",
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
