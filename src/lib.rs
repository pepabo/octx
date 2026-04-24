extern crate serde_urlencoded;

pub mod api_ext;
pub mod comments;
pub mod commits;
pub mod events;
pub mod issues;
pub mod labels;
pub mod pulls;
pub mod releases;
pub mod reviews;
pub mod users;
pub mod users_detailed;
pub mod workflows;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<octocrab::params::State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

impl Params {
    pub fn to_query(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            per_page: 100u8.into(),
            state: None,
            since: None,
            filter: None,
        }
    }
}

pub trait RepositryAware {
    fn set_repository(&mut self, name: String);
}

pub fn enum_to_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

// Drop scheme and authority from a pagination URI returned via GitHub's
// `Link` header so octocrab's execute() treats it as a relative request.
// GHES + App installation: octocrab only attaches the installation Bearer
// token when the request authority is empty or exactly `api.github.com`,
// so the absolute URL from `page.next` would otherwise be sent without
// Authorization and fail with 401/403 on page 2+.
pub fn to_relative_uri(uri: http::Uri) -> http::Uri {
    use std::str::FromStr;
    match uri.path_and_query() {
        Some(paq) => http::Uri::from_str(paq.as_str()).unwrap_or(uri),
        None => uri,
    }
}

pub trait UrlConstructor {
    fn reponame(&self) -> String;

    fn entrypoint_route(&self) -> String;
}

pub trait LoopWriter: UrlConstructor {
    type Model;
    type Record: serde::Serialize + RepositryAware + From<Self::Model>;

    fn write_and_continue<T: std::io::Write>(
        &self,
        mut page: octocrab::Page<Self::Model>,
        wtr: &mut csv::Writer<T>,
    ) -> Option<http::Uri> {
        let labels: Vec<Self::Model> = page.take_items();
        for label in labels.into_iter() {
            let mut label: Self::Record = label.into();
            label.set_repository(self.reponame());
            wtr.serialize(&label).expect("Serialize failed");
        }
        page.next.map(to_relative_uri)
    }
}
