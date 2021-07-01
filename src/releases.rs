use chrono::{DateTime, Utc};
use octocrab::models::repos::*;
use reqwest::Url;
use serde::*;

#[derive(Serialize, Debug)]
pub struct ReleaseRec {
    pub url: Url,
    pub html_url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    pub tarball_url: Url,
    pub zipball_url: Url,
    pub id: i64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    pub author_id: i64,
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
            author_id: from.author.id,
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
