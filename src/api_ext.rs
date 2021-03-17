use octocrab::{Octocrab, Page, Result};
use serde::*;

pub mod models {
    use reqwest::Url;
    use serde::*;
    type DateTime = chrono::DateTime<chrono::Utc>;

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

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct Run {
        pub id: i64,
        pub workflow_id: i64,
        pub node_id: String,
        pub name: String,
        pub head_branch: String,
        pub head_sha: String,
        pub run_number: i64,
        pub event: String,  // TODO: to_enum
        pub status: String, // TODO: to_enum
        #[serde(skip_serializing_if = "Option::is_none")]
        pub conclusion: Option<String>,
        pub created_at: DateTime,
        pub updated_at: DateTime,
        pub url: Url,
        pub html_url: Url,
        // TODO: other attrs
        // ref: https://docs.github.com/en/rest/reference/actions#list-workflow-runs
    }
}

pub struct WorkflowsHandler<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
}

impl<'octo> WorkflowsHandler<'octo> {
    pub fn new(crab: &'octo Octocrab, owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            crab,
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn list(&self) -> ListWorkflowsBuilder<'_, '_> {
        ListWorkflowsBuilder::new(self)
    }

    pub fn list_runs(&self, workflow_file: impl Into<Option<String>>) -> ListRunsBuilder<'_, '_> {
        ListRunsBuilder::new(self, workflow_file, None)
    }

    pub fn list_runs_by_id(&self, workflow_id: impl Into<Option<i64>>) -> ListRunsBuilder<'_, '_> {
        ListRunsBuilder::new(self, None, workflow_id)
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

    pub async fn send(self) -> Result<Page<models::WorkFlow>> {
        let url = format!(
            "repos/{owner}/{repo}/actions/workflows",
            owner = self.handler.owner,
            repo = self.handler.repo
        );
        self.handler.crab.get(url, Some(&self)).await
    }
}

#[derive(Serialize)]
pub struct ListRunsBuilder<'octo, 'b> {
    #[serde(skip)]
    handler: &'b WorkflowsHandler<'octo>,
    #[serde(skip)]
    workflow_file: Option<String>,
    #[serde(skip)]
    workflow_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u32>,
}

impl<'octo, 'b> ListRunsBuilder<'octo, 'b> {
    pub(crate) fn new(
        handler: &'b WorkflowsHandler<'octo>,
        workflow_file: impl Into<Option<String>>,
        workflow_id: impl Into<Option<i64>>,
    ) -> Self {
        Self {
            handler,
            workflow_file: workflow_file.into(),
            workflow_id: workflow_id.into(),
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

    pub async fn send(self) -> Result<Page<models::Run>> {
        let workflow_id = if let Some(file) = self.workflow_file.as_ref() {
            file.to_string()
        } else {
            format!("{}", self.workflow_id.unwrap())
        };

        let url = format!(
            "repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs",
            owner = self.handler.owner,
            repo = self.handler.repo,
            workflow_id = workflow_id
        );
        self.handler.crab.get(url, Some(&self)).await
    }
}

pub struct ListJobsBuilder<'octo> {
    crab: &'octo Octocrab,
    owner: String,
    repo: String,
    run_id: i64,
}
