use octocrab::models::*;
use reqwest::Url;
use serde::*;

use crate::*;

#[derive(Serialize, Debug)]
pub struct LabelRec {
    pub id: i64,
    pub node_id: String,
    pub url: Url,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub default: bool,

    pub sdc_repository: String,
}

impl RepositryAware for LabelRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<Label> for LabelRec {
    fn from(from: Label) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            name: from.name,
            description: from.description,
            color: from.color,
            default: from.default,

            sdc_repository: String::default(),
        }
    }
}

pub struct LabelFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

impl LabelFetcher {
    pub fn new(owner: String, name: String, octocrab: octocrab::Octocrab) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }
}

impl UrlConstructor for LabelFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self) -> Option<Url> {
        let mut param = Params::default();
        param.per_page = 100u8.into();

        let route = format!(
            "repos/{owner}/{repo}/labels?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for LabelFetcher {
    type Model = Label;
    type Record = LabelRec;
}

impl LabelFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}
