use super::*;

use octocrab::models::{issues, Author};
use url::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

// milestoned event should include only title
// See: https://docs.github.com/en/developers/webhooks-and-events/events/issue-event-types#milestoned
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MilestonePartial {
    pub title: String,
}

// Copied from octocrab::models::ProjectCard to fix null value
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectCard {
    pub id: u64,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_url: Option<Url>,
}

// Copied from octocrab::models::IssueEvent
// There are more events than Event enum defined
// Detail: https://docs.github.com/en/developers/webhooks-and-events/issue-event-types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigner: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_requester: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_reviewer: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<MilestonePartial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_card: Option<ProjectCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>, // Used instead of Event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue: issues::Issue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Debug)]
pub struct EventRec {
    pub id: Option<i64>,
    pub node_id: Option<String>,
    pub url: Option<String>,
    pub actor_id: Option<i64>,
    pub assignee_id: Option<i64>,
    pub assigner_id: Option<i64>,
    pub review_requester_id: Option<i64>,
    pub requested_reviewer_id: Option<i64>,
    pub label: Option<String>,
    pub milestone_title: Option<String>,
    pub project_card_url: Option<Url>,
    pub event: Option<String>, // Used instead of Event
    pub commit_id: Option<String>,
    pub commit_url: Option<Url>,
    pub created_at: DateTime,
    pub issue_id: i64,

    pub sdc_repository: String,
}

impl RepositryAware for EventRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl From<IssueEvent> for EventRec {
    fn from(from: IssueEvent) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            actor_id: from.actor.map(|u| u.id.0 as i64),
            event: from.event,
            assignee_id: from.assignee.map(|u| u.id.0 as i64),
            assigner_id: from.assigner.map(|u| u.id.0 as i64),
            review_requester_id: from.review_requester.map(|u| u.id.0 as i64),
            requested_reviewer_id: from.requested_reviewer.map(|u| u.id.0 as i64),
            label: from.label.map(|l| l.name),
            milestone_title: from.milestone.map(|m| m.title),
            project_card_url: from.project_card.map(|p| p.url),
            commit_id: from.commit_id,
            commit_url: from.commit_url,
            created_at: from.created_at,
            issue_id: from.issue.id.0 as i64,

            sdc_repository: String::default(),
        }
    }
}

pub struct IssueEventFetcher {
    owner: String,
    name: String,
    since: Option<DateTime>,
    octocrab: octocrab::Octocrab,
}

impl IssueEventFetcher {
    pub fn new(
        owner: String,
        name: String,
        since: Option<DateTime>,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            since,
            octocrab,
        }
    }
}

impl UrlConstructor for IssueEventFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint_route(&self) -> String {
        let param = Params::default();

        format!(
            "repos/{owner}/{repo}/issues/events?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        )
    }
}

impl LoopWriter for IssueEventFetcher {
    type Model = IssueEvent;
    type Record = EventRec;
}

impl IssueEventFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let first: octocrab::Page<IssueEvent> = self
            .octocrab
            .get(self.entrypoint_route(), None::<&()>)
            .await?;
        let mut page_opt = Some(first);

        while let Some(mut page) = page_opt {
            let labels: Vec<IssueEvent> = page.take_items();
            let mut last_update: Option<DateTime> = None;
            for label in labels.into_iter() {
                let mut label: EventRec = label.into();
                label.set_repository(self.reponame());
                wtr.serialize(&label).expect("Serialize failed");
                last_update = label.created_at.into()
            }
            let next = if let Some(since) = self.since {
                if last_update.unwrap() < since {
                    None
                } else {
                    page.next
                }
            } else {
                page.next
            };
            page_opt = self.octocrab.get_page(&next).await?;
        }

        Ok(())
    }
}
