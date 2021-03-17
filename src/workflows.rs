use csv::Writer;
use reqwest::Url;
use serde::*;
use serde_json;
type DateTime = chrono::DateTime<chrono::Utc>;
use crate::api_ext::models::*;
use crate::api_ext::*;

use std::ops::Deref;

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
    pub conclusion: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub url: Url,
    pub html_url: Url,

    pub sdc_repository: String,
}

#[derive(Serialize, Debug)]
pub struct JobRec {
    pub id: i64,
    pub run_id: i64,
    pub node_id: String,
    pub head_sha: String,
    pub status: String, // TODO: to_enum
    pub conclusion: Option<String>,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,
    pub name: String,
    pub url: Url,
    pub html_url: Url,
    pub run_url: Url,
    pub check_run_url: Url,
    pub steps: String,

    pub sdc_repository: String,
}

#[derive(Serialize, Debug)]
pub struct JobStepRec {
    pub job_id: i64,
    pub run_id: i64,
    pub job_node_id: String,
    pub head_sha: String,
    pub job_status: String, // TODO: to_enum
    pub job_conclusion: Option<String>,
    pub job_started_at: DateTime,
    pub job_completed_at: Option<DateTime>,
    pub job_name: String,
    pub job_url: Url,
    pub job_html_url: Url,
    pub run_url: Url,
    pub check_run_url: Url,

    pub name: String,
    pub status: String, // TODO: to_enum
    pub conclusion: Option<String>,
    pub number: i64,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,

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
            conclusion: from.conclusion,
            created_at: from.created_at,
            updated_at: from.updated_at,
            url: from.url,
            html_url: from.html_url,
            sdc_repository: String::default(),
        }
    }
}

impl From<Job> for JobRec {
    fn from(from: Job) -> Self {
        Self {
            id: from.id,
            run_id: from.run_id,
            node_id: from.node_id,
            head_sha: from.head_sha,
            status: from.status,
            conclusion: from.conclusion,
            started_at: from.started_at,
            completed_at: from.completed_at,
            name: from.name,
            url: from.url,
            html_url: from.html_url,
            run_url: from.run_url,
            check_run_url: from.check_run_url,
            steps: serde_json::to_string(&from.steps).unwrap_or("[]".to_string()),
            sdc_repository: String::default(),
        }
    }
}

impl From<Job> for Vec<JobStepRec> {
    fn from(from: Job) -> Vec<JobStepRec> {
        let mut ret = Vec::<JobStepRec>::new();
        for step in from.steps.into_iter() {
            ret.push(JobStepRec {
                job_id: from.id,
                run_id: from.run_id,
                job_node_id: from.node_id.clone(),
                head_sha: from.head_sha.clone(),
                job_status: from.status.clone(),
                job_conclusion: from.conclusion.clone(),
                job_started_at: from.started_at.clone(),
                job_completed_at: from.completed_at.clone(),
                job_name: from.name.clone(),
                job_url: from.url.clone(),
                job_html_url: from.html_url.clone(),
                run_url: from.run_url.clone(),
                check_run_url: from.check_run_url.clone(),

                name: step.name,
                status: step.status,
                conclusion: step.conclusion,
                number: step.number,
                started_at: step.started_at,
                completed_at: step.completed_at,

                sdc_repository: String::default(),
            });
        }
        ret
    }
}

pub struct WorkFlowFetcher<T: std::io::Write> {
    owner: String,
    name: String,
    workflow_id: Option<String>,
    octocrab: octocrab::Octocrab,
    writer: Writer<T>,
}

impl<T: std::io::Write> WorkFlowFetcher<T> {
    pub fn new(
        owner: String,
        name: String,
        workflow_id: Option<String>,
        octocrab: octocrab::Octocrab,
        writer: Writer<T>,
    ) -> Self {
        Self {
            owner,
            name,
            workflow_id,
            octocrab,
            writer,
        }
    }

    pub fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    pub async fn run(&mut self) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler.list().per_page(100).send().await?;

        let mut workflows: Vec<WorkFlow> = page.take_items();
        for workflow in workflows.drain(..) {
            let mut workflow: WorkFlowRec = workflow.into();
            workflow.sdc_repository = self.reponame();
            self.writer.serialize(&workflow).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            workflows.extend(newpage.take_items());
            for workflow in workflows.drain(..) {
                let mut workflow: WorkFlowRec = workflow.into();
                workflow.sdc_repository = self.reponame();
                self.writer.serialize(&workflow).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }

    pub async fn run_for_run(&mut self, workflow_id: impl Into<String>) -> octocrab::Result<()> {
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
            self.writer.serialize(&run).expect("Serialize failed");
        }
        while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
            runs.extend(newpage.take_items());
            for run in runs.drain(..) {
                let mut run: RunRec = run.into();
                run.sdc_repository = self.reponame();
                self.writer.serialize(&run).expect("Serialize failed");
            }
            page = newpage;
        }

        Ok(())
    }

    pub async fn run_for_all_run(&mut self) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler.list().per_page(100).send().await?;
        let workflows: Vec<i64> = page.take_items().into_iter().map(|item| item.id).collect();

        for wfid in workflows.into_iter() {
            let mut page = handler.list_runs_by_id(wfid).per_page(100).send().await?;

            let mut runs: Vec<Run> = page.take_items();
            for run in runs.drain(..) {
                let mut run: RunRec = run.into();
                run.sdc_repository = self.reponame();
                self.writer.serialize(&run).expect("Serialize failed");
            }
            while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
                runs.extend(newpage.take_items());
                for run in runs.drain(..) {
                    let mut run: RunRec = run.into();
                    run.sdc_repository = self.reponame();
                    self.writer.serialize(&run).expect("Serialize failed");
                }
                page = newpage;
            }
        }

        Ok(())
    }

    pub async fn run_for_job(
        &mut self,
        run_id: impl Into<i64>,
        denormalize_steps: bool,
    ) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler
            .list_jobs(run_id.into())
            .per_page(100)
            .send()
            .await?;

        let mut jobs: Vec<Job> = page.take_items();
        if denormalize_steps {
            for job in jobs.drain(..) {
                let steps: Vec<JobStepRec> = job.into();
                for mut step in steps.into_iter() {
                    step.sdc_repository = self.reponame();
                    self.writer.serialize(&step).expect("Serialize failed");
                }
            }
            while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
                jobs.extend(newpage.take_items());
                for job in jobs.drain(..) {
                    let steps: Vec<JobStepRec> = job.into();
                    for mut step in steps.into_iter() {
                        step.sdc_repository = self.reponame();
                        self.writer.serialize(&step).expect("Serialize failed");
                    }
                }
                page = newpage;
            }
        } else {
            for job in jobs.drain(..) {
                let mut job: JobRec = job.into();
                job.sdc_repository = self.reponame();
                self.writer.serialize(&job).expect("Serialize failed");
            }
            while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
                jobs.extend(newpage.take_items());
                for job in jobs.drain(..) {
                    let mut job: JobRec = job.into();
                    job.sdc_repository = self.reponame();
                    self.writer.serialize(&job).expect("Serialize failed");
                }
                page = newpage;
            }
        }

        Ok(())
    }

    pub async fn run_for_all_job(&mut self, denormalize_steps: bool) -> octocrab::Result<()> {
        let handler = WorkflowsHandler::new(&self.octocrab, &self.owner, &self.name);
        let mut page = handler.list().per_page(100).send().await?;
        let workflows: Vec<i64> = page.take_items().into_iter().map(|item| item.id).collect();

        let mut runs: Vec<i64> = Vec::new();
        for wfid in workflows.into_iter() {
            let mut page = handler.list_runs_by_id(wfid).per_page(100).send().await?;

            let items: Vec<Run> = page.take_items();
            for run in items.into_iter() {
                runs.push(run.id);
            }
            while let Some(mut newpage) = self.octocrab.get_page(&page.next).await? {
                let items: Vec<Run> = newpage.take_items();
                for run in items.into_iter() {
                    runs.push(run.id);
                }
                page = newpage;
            }
        }

        for run_id in runs.into_iter() {
            self.run_for_job(run_id, denormalize_steps).await?
        }

        Ok(())
    }
}
