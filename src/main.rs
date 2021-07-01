use anyhow::*;
use chrono::{DateTime, Duration, Utc};
use csv::WriterBuilder;
use log::*;
use serde::*;
use structopt::StructOpt;

use std::io;

extern crate octx;
use octx::{
    comments::CommentFetcher, commits::CommitFetcher, events::IssueEventFetcher,
    issues::IssueFetcher, labels::LabelFetcher, users::UserFetcher,
    users_detailed::UserDetailedFetcher,
};

#[derive(StructOpt)]
/// GitHub query & extracter (Enterprise ready)
#[structopt(after_help = "ENVIRONMENT VARIABLES:
    GITHUB_API_TOKEN: *Required* Token for your github. PAT is OK
    GITHUB_API_URL: Your enterprise server entrypoint. Please end the path with `/'

EXAMPLE:
    octx --issues rust-lang rust --days-ago 30
")]
struct Command {
    /// Extract issues - including pull requests
    #[structopt(long = "issues")]
    target_issues: bool,
    /// Extract issue events
    #[structopt(long = "events")]
    target_events: bool,
    /// Extract issue comments
    #[structopt(long = "comments")]
    target_comments: bool,
    /// Extract commits from a repo
    #[structopt(long = "commits")]
    target_commits: bool,
    /// Extract issue labels
    #[structopt(long = "labels")]
    target_labels: bool,
    /// Extract users with detailed info - owner/name is not required. this option takes some more minutes
    #[structopt(long = "users-detailed")]
    target_users_detailed: bool,
    /// Extract users - owner/name is not required for this option
    #[structopt(long = "users")]
    target_users: bool,
    /// Extract models created after N days ago.
    /// Only valid for --issues, --comments and --events
    #[structopt(long = "days-ago")]
    days_ago: Option<i64>,
    /// Extract models created after specified date.
    /// ISO 8601 format string is recommended.
    /// To see example, use e.g. `date --iso-8601=seconds`
    #[structopt(long = "since-date")]
    since_date: Option<String>,
    #[structopt(name = "owner")]
    owner: Option<String>,
    #[structopt(name = "name")]
    name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
    github_api_url: String,
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    env_logger::init();
    let config: Env = envy::from_env()
        .context("while reading from environment")
        .unwrap();
    let args = Command::from_args();
    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(config.github_api_token)
        .base_url(&config.github_api_url)?
        .build()?;

    let wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(io::stdout());

    if args.target_users_detailed {
        info!("Target: users(detailed)");
        let runner = UserDetailedFetcher::new(octocrab);
        runner.fetch(wtr).await?;
    } else if args.target_users {
        info!("Target: users");
        let runner = UserFetcher::new(octocrab);
        runner.fetch(wtr).await?;
    } else {
        let owner = args.owner.unwrap();
        let name = args.name.unwrap();

        let since = if args.days_ago.is_some() {
            args.days_ago.map(|ago| Utc::now() - Duration::days(ago))
        } else if args.since_date.is_some() {
            args.since_date
                .map(|date| DateTime::parse_from_rfc3339(&date).unwrap().into())
        } else {
            None
        };

        if args.target_issues {
            info!("Target: issues");
            let runner = IssueFetcher::new(owner, name, since, octocrab);
            runner.fetch(wtr).await?;
        } else if args.target_events {
            info!("Target: events");
            let runner = IssueEventFetcher::new(owner, name, since, octocrab);
            runner.fetch(wtr).await?;
        } else if args.target_comments {
            info!("Target: comments");
            let runner = CommentFetcher::new(owner, name, since, octocrab);
            runner.fetch(wtr).await?;
        } else if args.target_commits {
            info!("Target: commits");
            let runner = CommitFetcher::new(owner, name, since, octocrab);
            runner.fetch(wtr).await?;
        } else if args.target_labels {
            info!("Target: labels");
            let runner = LabelFetcher::new(owner, name, octocrab);
            runner.fetch(wtr).await?;
        } else {
            error!("No target specified");
        }
    }
    Ok(())
}
