use std::ops::{Deref, DerefMut};
use octocrab::models::workflows::{Job, WorkFlow};
use reqwest::Url;
use serde::Serialize;

use crate::*;
use crate::api_ext::models::*;

type DateTime = chrono::DateTime<chrono::Utc>;

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

#[derive(Serialize, Debug)]
pub struct JobStepsRec (Vec<JobStepRec>);

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

impl Deref for JobStepsRec {
    type Target = Vec<JobStepRec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for JobStepsRec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Job> for JobStepsRec {
    fn from(from: Job) -> JobStepsRec {
        let mut ret = JobStepsRec(Vec::<JobStepRec>::new());
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

impl RepositryAware for WorkFlowRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl RepositryAware for RunRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl RepositryAware for JobRec {
    fn set_repository(&mut self, name: String) {
        self.sdc_repository = name;
    }
}

impl RepositryAware for JobStepsRec {
    fn set_repository(&mut self, name: String) {
        for rec in self.iter_mut() {
            rec.sdc_repository = name.clone();
        }
    }
}

pub struct WorkFlowFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

pub struct RunFetcher {
    owner: String,
    name: String,
    octocrab: octocrab::Octocrab,
}

pub struct JobFetcher {
    owner: String,
    name: String,
    pub octocrab: octocrab::Octocrab,
}

pub struct JobStepFetcher {
    owner: String,
    name: String,
    job_id: String,
    pub octocrab: octocrab::Octocrab,
}

impl UrlConstructor for WorkFlowFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self) -> Option<Url> {
        let param = Params::default();

        let route = format!(
            "repos/{owner}/{repo}/actions/workflows?{query}",
            owner = &self.owner,
            repo = &self.name,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl UrlConstructor for JobStepFetcher {
    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self) -> Option<Url> {
        let param = Params::default();

        let route = format!(
            "repos/{owner}/{repo}/actions/jobs/{job_id}?{query}",
            owner = &self.owner,
            repo = &self.name,
            job_id = &self.job_id,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }
}

impl LoopWriter for WorkFlowFetcher {
    type Model = WorkFlow;
    type Record = WorkFlowRec;
}

impl LoopWriter for JobStepFetcher {
    type Model = Job;
    type Record = JobStepsRec;
}

impl WorkFlowFetcher {
    pub fn new(
        owner: String,
        name: String,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }

    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint();

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}

impl RunFetcher {
    pub fn new(
        owner: String,
        name: String,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }

    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self, workflow_id: Option<String>) -> Option<Url> {
        let param = Params::default();
        let route;

        if let Some(workflow_id_) = workflow_id {
            route = format!(
                "repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs?{query}",
                owner = &self.owner,
                repo = &self.name,
                workflow_id = &workflow_id_,
                query = param.to_query(),
            );
        } else {
            route = format!(
                "repos/{owner}/{repo}/actions/runs?{query}",
                owner = &self.owner,
                repo = &self.name,
                query = param.to_query(),
            )
        }

        self.octocrab.absolute_url(route).ok()
    }

    fn write_and_continue<T: std::io::Write>(
        &self,
        mut page: octocrab::Page<Run>,
        wtr: &mut csv::Writer<T>,
    ) -> Option<reqwest::Url> {
        let labels: Vec<Run> = page.take_items();
        for label in labels.into_iter() {
            let mut label: RunRec = label.into();
            label.set_repository(self.reponame());
            wtr.serialize(&label).expect("Serialize failed");
        }
        page.next
    }

    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>, workflow_id: Option<String>) -> octocrab::Result<()> {
        let mut next: Option<Url> = self.entrypoint(workflow_id);

        while let Some(page) = self.octocrab.get_page(&next).await? {
            next = self.write_and_continue(page, &mut wtr);
        }

        Ok(())
    }
}

impl JobFetcher {
    pub fn new(
        owner: String,
        name: String,
        octocrab: octocrab::Octocrab,
    ) -> Self {
        Self {
            owner,
            name,
            octocrab,
        }
    }

    fn reponame(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    fn entrypoint(&self, run_id: String) -> Option<Url> {
        let param = Params {
            filter: Some("all".to_string()),
            ..Default::default()
        };
        let route = format!(
            "repos/{owner}/{repo}/actions/runs/{run_id}/jobs?{query}",
            owner = &self.owner,
            repo = &self.name,
            run_id = &run_id,
            query = param.to_query(),
        );
        self.octocrab.absolute_url(route).ok()
    }

    fn write_and_continue<T: std::io::Write>(
        &self,
        mut page: octocrab::Page<Job>,
        wtr: &mut csv::Writer<T>,
    ) -> Option<reqwest::Url> {
        let labels: Vec<Job> = page.take_items();
        for label in labels.into_iter() {
            let mut label: JobRec = label.into();
            label.set_repository(self.reponame());
            wtr.serialize(&label).expect("Serialize failed");
        }
        page.next
    }

    pub async fn fetch<T: std::io::Write>(&self, mut wtr: csv::Writer<T>, run_id: Option<String>) -> octocrab::Result<()> {
        if let Some(run_id_) = run_id {
            let mut next: Option<Url> = self.entrypoint(run_id_);

            while let Some(page) = self.octocrab.get_page(&next).await? {
                next = self.write_and_continue(page, &mut wtr);
            }
        } else {
            let run_fetcher = RunFetcher::new(self.owner.clone(), self.name.clone(), self.octocrab.clone());
            for workflow in self.octocrab.workflows(&self.owner, &self.name).list().send().await? {
                let mut run_url = run_fetcher.entrypoint(Some(workflow.id.to_string()));
                while let Some(mut page) = self.octocrab.get_page(&run_url).await? {
                    let runs: Vec<Run> = page.take_items();
                    for run in runs.into_iter() {
                        let mut job_url: Option<Url> = self.entrypoint(run.id.to_string());
                        while let Some(page) = self.octocrab.get_page(&job_url).await? {
                            job_url = self.write_and_continue(page, &mut wtr);
                        }
                    }
                    run_url = page.next;
                }
            }
        }
                
        Ok(())
    }
}
