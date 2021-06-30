use chrono::{DateTime, Utc};
use octocrab::models::User;
use reqwest::Url;
use serde::*;

use crate::*;

// ref: https://docs.github.com/ja/rest/reference/users#get-a-user
// TODO: more more attributes to be required
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserDeailed {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: Url,
    pub gravatar_id: String,
    pub url: Url,
    pub html_url: Url,
    pub followers_url: Url,
    pub following_url: Url,
    pub gists_url: Url,
    pub starred_url: Url,
    pub subscriptions_url: Url,
    pub organizations_url: Url,
    pub repos_url: Url,
    pub events_url: Url,
    pub received_events_url: Url,
    pub r#type: String,
    pub site_admin: bool,

    pub name: Option<String>,
    pub email: Option<String>,
    pub twitter_username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserDetailedFetcher {
    octocrab: octocrab::Octocrab,
}

impl UserDetailedFetcher {
    pub fn new(octocrab: octocrab::Octocrab) -> Self {
        Self { octocrab }
    }
}

impl UserDetailedFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let param = Params::default();
        let route = format!("users?{query}", query = param.to_query());
        let mut next: Option<Url> = self.octocrab.absolute_url(route).ok();

        while let Some(mut page) = self.octocrab.get_page(&next).await? {
            let users: Vec<User> = page.take_items();
            for user in users.into_iter() {
                let detail: UserDeailed = self.octocrab.get(&user.url, None::<&()>).await?;
                wtr.serialize(&detail).expect("Serialize failed");
            }
            next = page.next
        }

        Ok(())
    }
}
