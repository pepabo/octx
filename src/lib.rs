extern crate serde_urlencoded;

pub mod comments;
pub mod commits;
pub mod events;
pub mod issues;
pub mod labels;
pub mod pulls;
pub mod releases;
pub mod users;
pub mod users_detailed;
pub mod workflows;
pub mod api_ext;

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

pub trait UrlConstructor {
    fn reponame(&self) -> String;

    fn entrypoint(&self) -> Option<reqwest::Url>;
}

pub trait LoopWriter: UrlConstructor {
    type Model;
    type Record: serde::Serialize + RepositryAware + From<Self::Model>;

    fn write_and_continue<T: std::io::Write>(
        &self,
        mut page: octocrab::Page<Self::Model>,
        wtr: &mut csv::Writer<T>,
    ) -> Option<reqwest::Url> {
        let labels: Vec<Self::Model> = page.take_items();
        for label in labels.into_iter() {
            let mut label: Self::Record = label.into();
            label.set_repository(self.reponame());
            wtr.serialize(&label).expect("Serialize failed");
        }
        page.next
    }
}
