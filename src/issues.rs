use octocrab::models::issues::*;
use octocrab::models::*;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Debug)]
pub struct IssueRec {
    pub id: i64,
    pub node_id: String,
    pub url: Url,
    pub repository_url: Url,
    pub labels_url: Url,
    pub comments_url: Url,
    pub events_url: Url,
    pub html_url: Url,
    pub number: i64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub user_id: i64,
    pub labels: String,
    pub assignee_id: Option<i64>,
    pub assignees: String,
    pub author_association: String,
    pub milestone: Option<String>,
    pub locked: bool,
    pub active_lock_reason: Option<String>,
    pub comments: u32,
    pub pull_request: Option<Url>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Issue> for IssueRec {
    fn from(from: Issue) -> IssueRec {
        let mut labels = from.labels;
        let mut assignees = from.assignees;

        IssueRec {
            id: from.id,
            node_id: from.node_id,
            url: from.url,
            repository_url: from.repository_url,
            labels_url: from.labels_url,
            comments_url: from.comments_url,
            events_url: from.events_url,
            html_url: from.html_url,
            number: from.number,
            state: from.state,
            title: from.title,
            body: from.body,
            body_text: from.body_text,
            body_html: from.body_html,
            user_id: from.user.id,
            labels: labels
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<String>>()
                .join(" "),
            assignee_id: match from.assignee {
                Some(user) => Some(user.id),
                None => None,
            },
            assignees: assignees
                .iter()
                .map(|v| v.login.clone())
                .collect::<Vec<String>>()
                .join(","),
            author_association: from.author_association,
            milestone: match from.milestone {
                Some(ms) => Some(ms.title),
                None => None,
            },
            locked: from.locked,
            active_lock_reason: from.active_lock_reason,
            comments: from.comments,
            pull_request: match from.pull_request {
                Some(pr) => Some(pr.url),
                None => None,
            },
            closed_at: from.closed_at,
            created_at: from.created_at,
            updated_at: from.updated_at,
        }
    }
}
