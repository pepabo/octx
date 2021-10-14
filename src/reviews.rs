extern crate octocrab;
use chrono::{DateTime, Utc};
use octocrab::models::User;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequest {
    pub number: u64,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

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
    pub state: Option<String>,
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
    pub state: Option<String>,
    pub pull_request_url: Option<Url>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub author_association: Option<String>,

    pub pull_request_number: Option<u64>,
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

pub struct ReviewFetcher {
    owner: String,
    name: String,
    since: Option<DateTime<Utc>>,
    octocrab: octocrab::Octocrab,
}

impl ReviewFetcher {
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

impl ReviewFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let param = Params::default();
        let route = format!(
            "repos/{owner}/{repo}/pulls?{query}&state=all&sort=updated&direction=desc",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        let mut next: Option<Url> = self.octocrab.absolute_url(route).ok();

        let mut pull_nums: Vec<u64> = vec![];
        while let Some(mut page) = self.octocrab.get_page(&next).await? {
            let pulls: Vec<PullRequest> = page.take_items();
            let mut last_update: Option<DateTime<Utc>> = None;
            for pull in pulls.into_iter() {
                pull_nums.push(pull.number);
                last_update = Some(pull.updated_at.unwrap_or_else(|| pull.created_at));
            }

            next = if let Some(since) = self.since {
                last_update.map_or_else(
                    || None,
                    |last| {
                        if last < since {
                            None
                        } else {
                            page.next
                        }
                    },
                )
            } else {
                page.next
            };
        }

        for number in pull_nums.into_iter() {
            let param = Params::default();
            let route = format!(
                "repos/{owner}/{repo}/pulls/{pull_number}/reviews?{query}",
                owner = &self.owner,
                repo = &self.name,
                pull_number = number,
                query = param.to_query(),
            );
            let mut next: Option<Url> = self.octocrab.absolute_url(route).ok();
            while let Some(mut page) = self.octocrab.get_page(&next).await? {
                let reviews: Vec<Review> = page.take_items();
                for review in reviews.into_iter() {
                    let mut review: ReviewRec = review.into();
                    review.sdc_repository = format!("{}/{}", self.owner, self.name);
                    review.pull_request_number = Some(number);

                    wtr.serialize(review).expect("Serialize failed");
                }
                next = page.next;
            }
        }
        Ok(())
    }
}
