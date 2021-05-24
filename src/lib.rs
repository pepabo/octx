extern crate serde_urlencoded;

pub mod comments;
pub mod events;
pub mod issues;
pub mod labels;
pub mod users;

use serde::Serialize;

#[derive(Serialize, Debug, Default)]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<octocrab::params::State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<chrono::DateTime<chrono::Utc>>,
}

impl Params {
    pub fn to_query(&self) -> String {
        serde_urlencoded::to_string(self).unwrap()
    }
}
