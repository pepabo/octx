use chrono::{DateTime, Utc};
use url::Url;
use serde::*;

//use crate::commits::{Commit, GitCommit, GitUser, Object, UserId};
use crate::commits::Commit;
use crate::*;
use octocrab::models::{AuthorAssociation, IssueState};

#[derive(Serialize, Debug)]
pub struct PullRequestRec {
    pub id: i64,
    pub number: u64,
    pub node_id: Option<String>,
    pub url: String,
    pub html_url: Option<String>,
    pub state: Option<IssueState>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub user_id: Option<i64>,
    pub user_login: Option<String>,
    pub user_type: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub merged: Option<bool>,
    pub merged_by_id: Option<i64>,
    pub merge_commit_sha: Option<String>,
    pub head_ref: String,
    pub head_sha: String,
    pub base_ref: String,
    pub base_sha: String,
    pub base_repo_archived: Option<bool>,
    pub base_repo_fork: Option<bool>,
    pub base_repo_default_branch: Option<String>,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub changed_files: Option<u64>,
    pub commits: Option<u64>,
    pub author_association: Option<AuthorAssociation>,
    pub draft: Option<bool>,
    pub locked: bool,
    pub maintainer_can_modify: bool,

    pub sdc_repository: String,
}

impl RepositryAware for PullRequestRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<octocrab::models::pulls::PullRequest> for PullRequestRec {
    fn from(p: octocrab::models::pulls::PullRequest) -> Self {
        let user_login = p.user.as_ref().map(|u| u.login.clone());
        let user_type = p.user.as_ref().map(|u| u.r#type.clone());
        let user_id = p.user.as_ref().map(|u| u.id.0 as i64);
        let base_repo_archived = p.base.repo.as_ref().and_then(|r| r.archived);
        let base_repo_fork = p.base.repo.as_ref().and_then(|r| r.fork);
        let base_repo_default_branch = p.base.repo.as_ref().and_then(|r| r.default_branch.clone());
        Self {
            id: p.id.0 as i64,
            number: p.number,
            node_id: p.node_id,
            url: p.url,
            html_url: p.html_url.map(|u| u.to_string()),
            state: p.state,
            title: p.title,
            body: p.body,
            user_id,
            user_login,
            user_type,
            created_at: p.created_at,
            updated_at: p.updated_at,
            closed_at: p.closed_at,
            merged_at: p.merged_at,
            merged: p.merged,
            merged_by_id: p.merged_by.map(|u| u.id.0 as i64),
            merge_commit_sha: p.merge_commit_sha,
            head_ref: p.head.ref_field.clone(),
            head_sha: p.head.sha.clone(),
            base_ref: p.base.ref_field.clone(),
            base_sha: p.base.sha.clone(),
            base_repo_archived,
            base_repo_fork,
            base_repo_default_branch,
            additions: p.additions,
            deletions: p.deletions,
            changed_files: p.changed_files,
            commits: p.commits,
            author_association: p.author_association,
            draft: p.draft,
            locked: p.locked,
            maintainer_can_modify: p.maintainer_can_modify,
            sdc_repository: String::default(),
        }
    }
}

pub struct PullsFetcher {
    owner: String,
    name: String,
    since: Option<DateTime<Utc>>,
    octocrab: octocrab::Octocrab,
}

impl PullsFetcher {
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

    fn pulls_route(&self) -> String {
        let param = Params::default();
        format!(
            "/repos/{owner}/{repo}/pulls?{query}&state=all&sort=updated&direction=desc",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        )
    }

    // additions / deletions / changed_files / commits は list endpoint では
    // 返らないため、 list で番号を引いた後に detail endpoint を 1 件ずつ叩いて
    // 完全な PR データを取得する。
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let first: octocrab::Page<octocrab::models::pulls::PullRequest> =
            self.octocrab.get(&self.pulls_route(), None::<&()>).await?;
        let mut page_opt = Some(first);

        while let Some(mut page) = page_opt {
            let pulls = page.take_items();
            let mut last_update: Option<DateTime<Utc>> = None;
            for pull_summary in pulls.into_iter() {
                let summary_updated = pull_summary.updated_at.or(pull_summary.created_at);
                last_update = summary_updated;

                // list endpoint の updated_at で since を満たさない PR は
                // detail を叩かずスキップする ( 増分取り込みで detail call 数を抑える ) 。
                // last_update は更新済みなのでループ末尾の打ち切り判定は引き続き機能する。
                if let Some(since) = self.since {
                    if summary_updated.map_or(false, |u| u < since) {
                        continue;
                    }
                }

                let detail_route = format!(
                    "/repos/{owner}/{repo}/pulls/{number}",
                    owner = &self.owner,
                    repo = &self.name,
                    number = pull_summary.number,
                );
                let pull: octocrab::models::pulls::PullRequest =
                    self.octocrab.get(&detail_route, None::<&()>).await?;

                let mut rec: PullRequestRec = pull.into();
                rec.set_repository(format!("{}/{}", self.owner, self.name));
                wtr.serialize(&rec).expect("Serialize failed");
            }

            let next = if let Some(since) = self.since {
                if last_update.is_some() && last_update.unwrap() < since {
                    None
                } else {
                    page.next.map(to_relative_uri)
                }
            } else {
                page.next.map(to_relative_uri)
            };
            page_opt = self.octocrab.get_page(&next).await?;
        }

        Ok(())
    }
}

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
    fn pulls_route(&self) -> String {
        let param = Params::default();
        format!(
            "/repos/{owner}/{repo}/pulls?{query}&state=all&sort=updated&direction=desc",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        )
    }

    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let first: octocrab::Page<PullRequest> =
            self.octocrab.get(&self.pulls_route(), None::<&()>).await?;
        let mut page_opt = Some(first);

        while let Some(mut page) = page_opt {
            let pulls: Vec<PullRequest> = page.take_items();
            let mut last_update: Option<DateTime<Utc>> = None;
            for pull in pulls.into_iter() {
                let files_route = format!(
                    "/repos/{owner}/{repo}/pulls/{number}/files",
                    owner = &self.owner,
                    repo = &self.name,
                    number = pull.number,
                );
                let mut files: Vec<PullRequestFile> =
                    self.octocrab.get(&files_route, None::<&()>).await?;
                for file in files.iter_mut() {
                    file.pull_request_number = pull.number.into();
                    file.sdc_repository = format!("{}/{}", self.owner, self.name).into();

                    wtr.serialize(file).expect("Serialize failed");
                }

                last_update = Some(pull.updated_at.unwrap_or_else(|| pull.created_at));
            }

            let next = if let Some(since) = self.since {
                if last_update.unwrap() < since {
                    None
                } else {
                    page.next.map(to_relative_uri)
                }
            } else {
                page.next.map(to_relative_uri)
            };
            page_opt = self.octocrab.get_page(&next).await?;
        }

        Ok(())
    }

    pub async fn fetch_commits<T: std::io::Write>(
        &self,
        mut wtr: csv::Writer<T>,
    ) -> octocrab::Result<()> {
        let first: octocrab::Page<PullRequest> =
            self.octocrab.get(&self.pulls_route(), None::<&()>).await?;
        let mut page_opt = Some(first);

        while let Some(mut page) = page_opt {
            let pulls: Vec<PullRequest> = page.take_items();
            let mut last_update: Option<DateTime<Utc>> = None;
            for pull in pulls.into_iter() {
                let commits_route = format!(
                    "/repos/{owner}/{repo}/pulls/{number}/commits",
                    owner = &self.owner,
                    repo = &self.name,
                    number = pull.number,
                );
                let commits: Vec<Commit> =
                    self.octocrab.get(&commits_route, None::<&()>).await?;
                for commit in commits.into_iter() {
                    let mut commit: PrCommitRec = commit.into();
                    commit.pull_request_number = pull.number.into();
                    commit.set_repository(format!("{}/{}", self.owner, self.name));

                    wtr.serialize(commit).expect("Serialize failed");
                }

                last_update = Some(pull.updated_at.unwrap_or_else(|| pull.created_at));
            }

            let next = if let Some(since) = self.since {
                if last_update.unwrap() < since {
                    None
                } else {
                    page.next.map(to_relative_uri)
                }
            } else {
                page.next.map(to_relative_uri)
            };
            page_opt = self.octocrab.get_page(&next).await?;
        }

        Ok(())
    }
}
