use super::*;

use octocrab::models::{issues, Milestone, ProjectCard, User};
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

// Copied from octocrab::models::IssueEvent
// There are more events than Event enum defined
// Detailed: https://docs.github.com/en/developers/webhooks-and-events/issue-event-types
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigner: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_requester: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_reviewer: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Milestone>,
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
    pub actor_id: i64,
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
            actor_id: from.actor.id,
            event: from.event,
            assignee_id: from.assignee.map(|u| u.id),
            assigner_id: from.assigner.map(|u| u.id),
            review_requester_id: from.review_requester.map(|u| u.id),
            requested_reviewer_id: from.requested_reviewer.map(|u| u.id),
            label: from.label.map(|l| l.name),
            milestone_title: from.milestone.map(|m| m.title),
            project_card_url: from.project_card.map(|p| p.url),
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

    fn entrypoint(&self) -> Option<Url> {
        let mut param = Params::default();
        param.per_page = 100u8.into();

        let route = format!(
            "repos/{owner}/{repo}/issues/events?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for IssueEventFetcher {
    type Model = IssueEvent;
    type Record = EventRec;
}

impl IssueEventFetcher {
    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(mut page) = self.octocrab.get_page(&next).await? {
            let labels: Vec<IssueEvent> = page.take_items();
            let mut last_update: Option<DateTime> = None;
            for label in labels.into_iter() {
                let mut label: EventRec = label.into();
                label.set_repository(self.reponame());
                wtr.serialize(&label).expect("Serialize failed");
                last_update = label.created_at.into()
            }
            next = if let Some(since) = self.since {
                if last_update.unwrap() < since {
                    None
                } else {
                    page.next
                }
            } else {
                page.next
            };
        }

        Ok(())
    }
}
