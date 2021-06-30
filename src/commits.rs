use octocrab::models::{repos::Object, User};
use reqwest::Url;
use serde::*;

use crate::*;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Commit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committer: Option<User>,
    pub commit: GitCommit,
    pub parents: Vec<Object>, // is added
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitCommit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<GitUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committer: Option<GitUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub comment_count: i32,
    // pub verification
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitUser {
    pub name: String,
    pub email: String,
    pub date: DateTime, // is required
}

#[derive(Serialize, Debug)]
pub struct CommitRec {
    pub sha: Option<String>,
    pub node_id: Option<String>,
    pub url: Option<String>,
    pub html_url: Option<String>,
    pub comments_url: Option<String>,
    pub author_id: Option<i64>,
    pub committer_id: Option<i64>,
    pub parents: String, // Vec.to_json
    pub message: Option<String>,
    pub authorized_at: Option<DateTime>,
    pub committed_at: Option<DateTime>,
    pub comment_count: i32,

    pub sdc_repository: String,
}

impl RepositryAware for CommitRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<Commit> for CommitRec {
    fn from(from: Commit) -> Self {
        Self {
            sha: from.sha,
            node_id: from.node_id,
            url: from.url,
            html_url: from.html_url,
            comments_url: from.comments_url,
            author_id: from.author.map(|u| u.id),
            committer_id: from.committer.map(|u| u.id),
            parents: from
                .parents
                .iter()
                .map(|v| match v {
                    Object::Commit { sha, url: _ } => sha.to_owned(),
                    _ => "".to_string(),
                })
                .collect::<Vec<String>>()
                .join(" "),
            message: from.commit.message,
            authorized_at: from.commit.author.map(|a| a.date),
            committed_at: from.commit.committer.map(|a| a.date),
            comment_count: from.commit.comment_count,

            sdc_repository: String::default(),
        }
    }
}

pub struct CommitFetcher {
    owner: String,
    name: String,
    since: Option<DateTime>,
    octocrab: octocrab::Octocrab,
}

impl CommitFetcher {
    pub fn new(
        owner: String,
        name: String,
        since: Option<DateTime>,
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

impl UrlConstructor for CommitFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self) -> Option<Url> {
        let param = Params {
            since: self.since,
            ..Default::default()
        };

        let route = format!(
            "repos/{owner}/{repo}/commits?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for CommitFetcher {
    type Model = Commit;
    type Record = CommitRec;
}

impl CommitFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}
