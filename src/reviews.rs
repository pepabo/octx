extern crate octocrab;
use octocrab::models::{pulls::ReviewState, User};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::*;

// Should support author_association: key.
// ref: https://docs.github.com/en/rest/reference/pulls#list-reviews-for-a-pull-request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Review {
    pub id: u64,
    pub node_id: String,
    pub html_url: Url,
    pub user: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<ReviewState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    // omits links for our use case, for now
    // #[serde(rename = "_links")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub links: Option<Links>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_association: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewRec {
    pub id: u64,
    pub node_id: String,
    pub html_url: Url,
    pub user_id: i64,
    pub body: Option<String>,
    pub commit_id: Option<String>,
    pub state: Option<ReviewState>,
    pub pull_request_url: Option<Url>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub author_association: Option<String>,

    pub pull_request_number: Option<i64>,
    pub sdc_repository: String,
}

impl RepositryAware for ReviewRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<Review> for ReviewRec {
    fn from(from: Review) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            html_url: from.html_url,
            user_id: from.user.id,
            body: from.body,
            commit_id: from.commit_id,
            state: from.state,
            pull_request_url: from.pull_request_url,
            submitted_at: from.submitted_at,
            author_association: from.author_association,

            pull_request_number: None,
            sdc_repository: String::default(),
        }
    }
}
