use csv::Writer;
use octocrab::models::*;
use reqwest::Url;
use serde::*;

use crate::Params;

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

    pub fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub async fn run<T: std::io::Write>(&self, mut wtr: Writer<T>) -> octocrab::Result<()> {
        let mut param = Params::default();
        param.per_page = 100u8.into();

        let route = format!(
            "repos/{owner}/{repo}/labels?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        let mut next: Option<Url> = self.octocrab.absolute_url(route).ok();

        while let Some(mut page) = self.octocrab.get_page(&next).await? {
            let labels: Vec<Label> = page.take_items();
            for label in labels.into_iter() {
                let mut label: LabelRec = label.into();
                label.sdc_repository = self.reponame();
                wtr.serialize(&label).expect("Serialize failed");
            }
            next = page.next;
        }

        Ok(())
    }
}
