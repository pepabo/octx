use csv::Writer;
use octocrab::models::{issues, User};
use octocrab::Page;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

// Copied from octocrab::models::IssueEvent
// There are more events than Event enum defined
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub actor: User,
    pub event: Option<String>, // Used instead of Event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue: issues::Issue,
}

#[derive(Serialize, Debug)]
struct EventRec {
    pub id: Option<i64>,
    pub node_id: Option<String>,
    pub url: Option<String>,
    pub actor_id: i64,
    pub event: Option<String>, // Used instead of Event
    pub commit_id: Option<String>,
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue_id: i64,

    pub sdc_repository: String,
}

impl From<IssueEvent> for EventRec {
    fn from(from: IssueEvent) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            actor_id: from.actor.id,
            event: from.event,
            commit_id: from.commit_id,
            commit_url: from.commit_url,
            created_at: from.created_at,
            issue_id: from.issue.id,

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

        let mut page: Page<IssueEvent> = self.octocrab.get(route, Some(&handler)).await?;

        let mut events: Vec<IssueEvent> = page.take_items();
        for event in events.drain(..) {
            let mut event: EventRec = event.into();
            event.sdc_repository = self.reponame();
            wtr.serialize(&event).expect("Serialize failed");
        }

        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            let mut events: Vec<IssueEvent> = newpage.take_items();
            for event in events.drain(..) {
                let mut event: EventRec = event.into();
                event.sdc_repository = self.reponame();
                wtr.serialize(&event).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}
