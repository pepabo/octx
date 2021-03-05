use csv::Writer;
use octocrab::models::User;
use octocrab::params;
use octocrab::Page;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

pub struct UserFetcher {
    octocrab: octocrab::Octocrab,
}

#[derive(Serialize)]
struct UserHandler {
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

#[derive(Serialize, Debug)]
pub struct UserRec {
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
}

impl From<User> for UserRec {
    fn from(from: User) -> UserRec {
        Self {
            login: from.login,
            id: from.id,
            node_id: from.node_id,
            avatar_url: from.avatar_url,
            gravatar_id: from.gravatar_id,
            url: from.url,
            html_url: from.html_url,
            followers_url: from.followers_url,
            following_url: from.following_url,
            gists_url: from.gists_url,
            starred_url: from.starred_url,
            subscriptions_url: from.subscriptions_url,
            organizations_url: from.organizations_url,
            repos_url: from.repos_url,
            events_url: from.events_url,
            received_events_url: from.received_events_url,
            r#type: from.r#type,
            site_admin: from.site_admin,
        }
    }
}

impl UserFetcher {
    pub fn new(octocrab: octocrab::Octocrab) -> Self {
        Self { octocrab }
    }

    pub async fn run<T: std::io::Write>(&self, mut wtr: Writer<T>) -> octocrab::Result<()> {
        let handler = UserHandler {
            per_page: Some(100u8),
            page: None,
        };
        let mut page: Page<User> = self.octocrab.get("users", Some(&handler)).await?;

        let mut users: Vec<User> = page.take_items();
        for user in users.drain(..) {
            let user: UserRec = user.into();
            wtr.serialize(&user).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            users.extend(newpage.take_items());
            for user in users.drain(..) {
                let user: UserRec = user.into();
                wtr.serialize(&user).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}
