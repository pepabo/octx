use csv::Writer;
use reqwest::Url;
use serde::*;
type DateTime = chrono::DateTime<chrono::Utc>;
use crate::api_ext::models::*;
use crate::api_ext::*;

#[derive(Serialize, Debug)]
pub struct WorkFlowRec {
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

    pub sdc_repository: String,
}

#[derive(Serialize, Debug)]
pub struct RunRec {
    pub id: i64,
    pub workflow_id: i64,
    pub node_id: String,
    pub name: String,
    pub head_branch: String,
    pub head_sha: String,
    pub run_number: i64,
    pub event: String,  // TODO: to_enum
    pub status: String, // TODO: to_enum
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub url: Url,
    pub html_url: Url,

    pub sdc_repository: String,
}

impl From<WorkFlow> for WorkFlowRec {
    fn from(from: WorkFlow) -> Self {
        Self {
            id: from.id,
            node_id: from.node_id,
            name: from.name,
            path: from.path,
            state: from.state,
            created_at: from.created_at,
            updated_at: from.updated_at,
            url: from.url,
            html_url: from.html_url,
            badge_url: from.badge_url,
            sdc_repository: String::default(),
        }
    }
}

impl From<Run> for RunRec {
    fn from(from: Run) -> Self {
        Self {
            id: from.id,
            workflow_id: from.workflow_id,
            node_id: from.node_id,
            name: from.name,
            head_branch: from.head_branch,
            head_sha: from.head_sha,
            run_number: from.run_number,
            event: from.event,
            status: from.status,
            created_at: from.created_at,
            updated_at: from.updated_at,
            url: from.url,
            html_url: from.html_url,
            sdc_repository: String::default(),
        }
    }
}

pub struct WorkFlowFetcher {
    owner: String,
    name: String,
    workflow_id: Option<String>,
    octocrab: octocrab::Octocrab,
}

impl WorkFlowFetcher {
    pub fn new(
        owner: String,
        name: String,
        workflow_id: Option<String>,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            workflow_id,
            octocrab,
        }
    }

    pub fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub async fn run<T: std::io::Write>(&self, mut wtr: Writer<T>) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler.list().per_page(100).send().await?;

        let mut workflows: Vec<WorkFlow> = page.take_items();
        for workflow in workflows.drain(..) {
            let mut workflow: WorkFlowRec = workflow.into();
            workflow.sdc_repository = self.reponame();
            wtr.serialize(&workflow).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            workflows.extend(newpage.take_items());
            for workflow in workflows.drain(..) {
                let mut workflow: WorkFlowRec = workflow.into();
                workflow.sdc_repository = self.reponame();
                wtr.serialize(&workflow).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }

    pub async fn run_for_run<T: std::io::Write>(
        &self,
        workflow_id: impl Into<String>,
        mut wtr: Writer<T>,
    ) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler
            .list_runs(workflow_id.into())
            .per_page(100)
            .send()
            .await?;

        let mut runs: Vec<Run> = page.take_items();
        for run in runs.drain(..) {
            let mut run: RunRec = run.into();
            run.sdc_repository = self.reponame();
            wtr.serialize(&run).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            runs.extend(newpage.take_items());
            for run in runs.drain(..) {
                let mut run: RunRec = run.into();
                run.sdc_repository = self.reponame();
                wtr.serialize(&run).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }
}
