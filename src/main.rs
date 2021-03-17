use anyhow::*;
use chrono::{Duration, Utc};
use csv::WriterBuilder;
use log::*;
use serde::*;
use structopt::StructOpt;

use std::io;

mod issues;
use issues::IssueFetcher;
mod comments;
use comments::CommentFetcher;
mod labels;
use labels::LabelFetcher;
mod users;
use users::UserFetcher;
mod workflows;
use workflows::WorkFlowFetcher;
mod api_ext;
use api_ext::*;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(long = "issues")]
    target_issues: bool,
    #[structopt(long = "comments")]
    target_comments: bool,
    #[structopt(long = "labels")]
    target_labels: bool,
    #[structopt(long = "users")]
    target_users: bool,
    #[structopt(long = "workflows")]
    target_workflows: bool,
    #[structopt(long = "runs")]
    target_runs: bool,
    #[structopt(long = "jobs")]
    target_jobs: bool,
    #[structopt(long = "denormalize-steps")]
    denormalize_steps: bool,
    #[structopt(long = "days-ago")]
    days_ago: Option<i64>,
    #[structopt(long = "owner", default_value = "")]
    owner: String,
    #[structopt(long = "name", default_value = "")]
    name: String,
    #[structopt(long = "workflow-file")]
    workflow_file: Option<String>,
    #[structopt(long = "run-id")]
    run_id: Option<i64>,
}

#[derive(Debug, Clone)]
struct OptionInvalid(String);
impl std::fmt::Display for OptionInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OptionErro: {}", self.0)
    }
}

impl std::error::Error for OptionInvalid {}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
    github_api_url: String,
}

#[tokio::main]
async fn main() -> octocrab::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config: Env = envy::from_env()
        .context("while reading from environment")
        .unwrap();
    let args = Command::from_args();
    let owner = args.owner;
    let name = args.name;

    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(config.github_api_token)
        .base_url(&config.github_api_url)?
        .build()?;

    let wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(io::stdout());

    if args.target_issues {
        info!("Target: issues");
        let runner = IssueFetcher::new(owner, name, octocrab);
        runner.run(wtr).await?;
    } else if args.target_comments {
        info!("Target: comments");
        let runner = CommentFetcher::new(owner, name, octocrab);
        let since = args.days_ago.map(|ago| Utc::now() - Duration::days(ago));
        runner.run(since, wtr).await?;
    } else if args.target_labels {
        info!("Target: labes");
        let runner = LabelFetcher::new(owner, name, octocrab);
        runner.run(wtr).await?;
    } else if args.target_users {
        info!("Target: labes");
        let runner = UserFetcher::new(octocrab);
        runner.run(wtr).await?;
    } else if args.target_workflows {
        info!("Target: workflows");
        let mut runner = WorkFlowFetcher::new(owner, name, None, octocrab, wtr);
        runner.run().await?;
    } else if args.target_runs {
        info!("Target: runs");
        if let Some(worklow_id) = args.workflow_file {
            let mut runner = WorkFlowFetcher::new(owner, name, None, octocrab, wtr);
            runner.run_for_run(worklow_id).await?;
        } else {
            let mut runner = WorkFlowFetcher::new(owner, name, None, octocrab, wtr);
            runner.run_for_all_run().await?;
        }
    } else if args.target_jobs {
        info!("Target: jobs");
        let mut runner = WorkFlowFetcher::new(owner, name, None, octocrab, wtr);
        if let Some(run_id) = args.run_id {
            runner.run_for_job(run_id, args.denormalize_steps).await?;
        } else {
            runner.run_for_all_job(args.denormalize_steps).await?;
        }
    } else {
        error!("No target specified");
    }
    Ok(())
}
