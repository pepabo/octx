use octocrab::{Octocrab, Page, Result};
use serde::*;

pub mod models {
    use reqwest::Url;
    use serde::*;
    type DateTime = chrono::DateTime<chrono::Utc>;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct WorkFlows {
        pub total_count: u64,
        pub workflows: Vec<WorkFlow>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct WorkFlow {
        pub id: i64,
        pub node_id: String,
        pub name: String,
        pub path: String,
        pub state: String, // TODO: to_enum
        pub created_at: DateTime,
        pub updated_at: DateTime,
        pub url: Url,
        pub html_url: Url,
        pub badge_url: Url,
    }
}

pub struct WorkflowsHandler<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
}

impl<'octo> WorkflowsHandler<'octo> {
    pub fn new(crab: &'octo Octocrab, owner: String, repo: String) -> Self {
        Self { crab, owner, repo }
    }

    pub fn list(&self) -> ListWorkflowsBuilder<'_, '_> {
        ListWorkflowsBuilder::new(self)
    }

    pub fn list_runs(&self, workflow_id: i64) -> ! {
        todo!("Runs: https://docs.github.com/en/rest/reference/actions#list-workflow-runs")
    }

    pub fn list_jobs(&self, run_id: i64) -> ! {
        todo!(
            "Jobs: https://docs.github.com/en/rest/reference/actions#list-jobs-for-a-workflow-run"
        )
    }
}

#[derive(Serialize)]
pub struct ListWorkflowsBuilder<'octo, 'b> {
    #[serde(skip)]
    handler: &'b WorkflowsHandler<'octo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

impl<'octo, 'b> ListWorkflowsBuilder<'octo, 'b> {
    pub(crate) fn new(handler: &'b WorkflowsHandler<'octo>) -> Self {
        Self {
            handler,
            per_page: None,
            page: None,
        }
    }

    pub fn per_page(mut self, per_page: impl Into<u8>) -> Self {
        self.per_page = Some(per_page.into());
        self
    }

    pub fn page(mut self, page: impl Into<u32>) -> Self {
        self.page = Some(page.into());
        self
    }

    pub async fn send(self) -> Result<Page<models::WorkFlows>> {
        let url = format!(
            "repos/{owner}/{repo}/actions/workflows",
            owner = self.handler.owner,
            repo = self.handler.repo
        );
        self.handler.crab.get(url, Some(&self)).await
    }
}

pub struct ListRunsBuilder<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
    workflow_id: i64,
}

pub struct ListJobsBuilder<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
    run_id: i64,
}
