use octocrab::models::issues::*;
use url::Url;
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
            id: from.id.0,
            node_id: from.node_id,
            url: from.url,
            html_url: from.html_url,
            body: from.body,
            body_text: from.body_text,
            body_html: from.body_html,
            user_id: from.user.id.0 as i64,
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

    fn entrypoint_route(&self) -> String {
        let param = Params {
            since: self.since,
            ..Default::default()
        };

        format!(
            "/repos/{owner}/{repo}/issues/comments?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        )
    }
}

impl LoopWriter for CommentFetcher {
    type Model = Comment;
    type Record = CommentRec;
}

impl CommentFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let first: octocrab::Page<Comment> = self
            .octocrab
            .get(self.entrypoint_route(), None::<&()>)
            .await?;
        let mut next = self.write_and_continue(first, &mut wtr);

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}
