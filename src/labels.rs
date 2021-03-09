use csv::Writer;
use octocrab::models::*;
use reqwest::Url;
use serde::*;

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
        let mut page = self
            .octocrab
            .issues(&self.owner, &self.name)
            .list_labels_for_repo()
            .per_page(100)
            .send()
            .await?;

        let mut labels: Vec<Label> = page.take_items();
        for label in labels.drain(..) {
            let mut label: LabelRec = label.into();
            label.sdc_repository = self.reponame();
            wtr.serialize(&label).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            labels.extend(newpage.take_items());
            for label in labels.drain(..) {
                let mut label: LabelRec = label.into();
                label.sdc_repository = self.reponame();
                wtr.serialize(&label).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}
