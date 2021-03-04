use anyhow::*;
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

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(long = "issues")]
    target_issues: bool,
    #[structopt(long = "comments")]
    target_comments: bool,
    #[structopt(long = "labels")]
    target_labels: bool,
    #[structopt(long = "owner")]
    owner: String,
    #[structopt(long = "name")]
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
        runner.run(wtr).await?;
    } else if args.target_comments {
        info!("Target: comments");
        let runner = CommentFetcher::new(owner, name, octocrab);
        runner.run(wtr).await?;
    } else if args.target_labels {
        info!("Target: labes");
        let runner = LabelFetcher::new(owner, name, octocrab);
        runner.run(wtr).await?;
    } else {
        error!("No target specified");
    }
    Ok(())
}
