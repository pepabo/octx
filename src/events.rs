use csv::Writer;
// use octocrab::models::events::*;
use octocrab::Page;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

/// A GitHub event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
struct Event {
    pub id: i64,
    pub event: String,
}

#[derive(Serialize, Debug)]
struct EventRec {
    pub id: i64,

    pub sdc_repository: String,
}

impl From<Event> for EventRec {
    fn from(from: Event) -> Self {
        Self {
            id: from.id,

            sdc_repository: String::default(),
        }
    }
}

pub struct IssueEventFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

#[derive(Serialize)]
struct EventHandler {
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

impl IssueEventFetcher {
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
        let handler = EventHandler {
            per_page: Some(100u8),
            page: None,
        };
        let route = format!(
            "repos/{owner}/{repo}/issues/events",
            owner = &self.owner,
            repo = &self.name,
        );

        let mut page: Page<Event> = self.octocrab.get(route, Some(&handler)).await?;

        let events: Vec<Event> = page.take_items();
        println!("{:?}", events);

        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            let events: Vec<Event> = newpage.take_items();
            println!("{:?}", events);
            page = newpage;
        }

        Ok(())
    }
}
