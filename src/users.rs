use octocrab::models::User;
use reqwest::Url;
use serde::*;

use crate::*;

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

impl RepositryAware for UserRec {
    fn set_repository(&mut self, _: String) {}
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

pub struct UserFetcher {
    octocrab: octocrab::Octocrab,
}

impl UserFetcher {
    pub fn new(octocrab: octocrab::Octocrab) -> Self {
        Self { octocrab }
    }
}

impl UrlConstructor for UserFetcher {
    fn reponame(&self) -> String {
        "".to_string()
    }

    fn entrypoint(&self) -> Option<Url> {
        let param = Params::default();

        let route = format!("users?{query}", query = param.to_query());
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for UserFetcher {
    type Model = User;
    type Record = UserRec;
}

impl UserFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}
