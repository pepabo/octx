use anyhow::*;
use chrono::{Duration, Utc};
use csv::WriterBuilder;
use log::*;
use serde::*;
use structopt::StructOpt;

use std::io;

extern crate octx;
use octx::{
    comments::CommentFetcher, events::IssueEventFetcher, issues::IssueFetcher,
    labels::LabelFetcher, users::UserFetcher,
};

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(long = "issues")]
    target_issues: bool,
    #[structopt(long = "events")]
    target_events: bool,
    #[structopt(long = "comments")]
    target_comments: bool,
    #[structopt(long = "labels")]
    target_labels: bool,
    #[structopt(long = "users")]
    target_users: bool,
    #[structopt(long = "days-ago")]
    days_ago: Option<i64>,
    #[structopt(long = "owner", default_value = "")]
    owner: String,
    #[structopt(long = "name", default_value = "")]
    name: String,
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
        runner.fetch(wtr).await?;
    } else if args.target_events {
        info!("Target: events");
        let runner = IssueEventFetcher::new(owner, name, octocrab);
        runner.run(wtr).await?;
    } else if args.target_comments {
        info!("Target: comments");
        let runner = CommentFetcher::new(owner, name, octocrab);
        let since = args.days_ago.map(|ago| Utc::now() - Duration::days(ago));
        runner.run(since, wtr).await?;
    } else if args.target_labels {
        info!("Target: labes");
        let runner = LabelFetcher::new(owner, name, octocrab);
        runner.fetch(wtr).await?;
    } else if args.target_users {
        info!("Target: labes");
        let runner = UserFetcher::new(octocrab);
        runner.run(wtr).await?;
    } else {
        error!("No target specified");
    }
    Ok(())
}
