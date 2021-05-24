use octocrab::models::issues::*;
use reqwest::Url;
use serde::*;

use crate::*;

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Debug)]
pub struct CommentRec {
    pub id: u64,
    pub node_id: String,
    pub url: Url,
    pub html_url: Url,
    pub body: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub user_id: i64,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,

    pub sdc_repository: String,
}

impl RepositryAware for CommentRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<Comment> for CommentRec {
    fn from(from: Comment) -> CommentRec {
        CommentRec {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            html_url: from.html_url,
            body: from.body,
            body_text: from.body_text,
            body_html: from.body_html,
            user_id: from.user.id,
            created_at: from.created_at,
            updated_at: from.updated_at,

            sdc_repository: String::default(),
        }
    }
}

pub struct CommentFetcher {
    owner: String,
    name: String,
    since: Option<DateTime>,
    octocrab: octocrab::Octocrab,
}

impl CommentFetcher {
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

impl UrlConstructor for CommentFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self) -> Option<Url> {
        let mut param = Params::default();
        param.per_page = 100u8.into();
        param.since = self.since;

        let route = format!(
            "repos/{owner}/{repo}/issues/comments?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for CommentFetcher {
    type Model = Comment;
    type Record = CommentRec;
}

impl CommentFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}
