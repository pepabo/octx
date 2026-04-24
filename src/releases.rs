use chrono::{DateTime, Utc};
use octocrab::models::repos::Asset;
use url::Url;
use serde::*;

use crate::*;

// use octocrab::models::repos::Release;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Release {
    pub url: Url,
    pub html_url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    pub tarball_url: Option<Url>,
    pub zipball_url: Option<Url>,
    pub id: i64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<octocrab::models::Author>,
    pub assets: Vec<Asset>,
}

#[derive(Serialize, Debug)]
pub struct ReleaseRec {
    pub url: Url,
    pub html_url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    pub tarball_url: Option<Url>,
    pub zipball_url: Option<Url>,
    pub id: i64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author_id: Option<i64>,
    pub assets: String,

    pub sdc_repository: String,
}

impl From<Release> for ReleaseRec {
    fn from(from: Release) -> Self {
        Self {
            url: from.url,
            html_url: from.html_url,
            assets_url: from.assets_url,
            upload_url: from.upload_url,
            tarball_url: from.tarball_url,
            zipball_url: from.zipball_url,
            id: from.id,
            node_id: from.node_id,
            tag_name: from.tag_name,
            target_commitish: from.target_commitish,
            name: from.name,
            body: from.body,
            draft: from.draft,
            prerelease: from.prerelease,
            created_at: from.created_at,
            published_at: from.published_at,
            author_id: from.author.map(|u| u.id.0 as i64),
            assets: from
                .assets
                .iter()
                .map(|v| format!("{};{};{}", v.id, v.name, v.browser_download_url))
                .collect::<Vec<String>>()
                .join(","),

            sdc_repository: String::default(),
        }
    }
}

impl RepositryAware for ReleaseRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

pub struct ReleaseFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

impl ReleaseFetcher {
    pub fn new(owner: String, name: String, octocrab: octocrab::Octocrab) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }
}

impl UrlConstructor for ReleaseFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint_route(&self) -> String {
        let param = Params::default();

        format!(
            "repos/{owner}/{repo}/releases?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        )
    }
}

impl LoopWriter for ReleaseFetcher {
    type Model = Release;
    type Record = ReleaseRec;
}

impl ReleaseFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let first: octocrab::Page<Release> = self
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
