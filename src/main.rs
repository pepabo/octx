use anyhow::*;
use csv::WriterBuilder;
use log::*;
use octocrab::{models, params};
use serde::*;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(name = "owner")]
    owner: String,
    #[structopt(name = "name")]
    name: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
    github_api_url: String,
}

mod issues;
use issues::IssueRec;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    env_logger::init();
    let config: Env = envy::from_env()
        .context("while reading from environment")
        .unwrap();
    let args = Command::from_args();
    let owner = args.owner;
    let repo = args.name;

    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(config.github_api_token)
        .base_url(&config.github_api_url)?
        .build()?;

    let mut page = octocrab
        .issues(owner, repo)
        .list()
        .state(params::State::All)
        .per_page(100)
        .send()
        .await?;

    let mut issues: Vec<models::issues::Issue> = page.take_items();
    while let Some(mut newpage) = octocrab.get_page(&page.next).await? {
        issues.extend(newpage.take_items());
        page = newpage;
    }

    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);

    for issue in issues.drain(..) {
        let issue: IssueRec = issue.into();
        wtr.serialize(&issue).expect("Serialize failed");
    }
    let data = wtr.into_inner().expect("Intodata failed");
    println!(
        "{}",
        String::from_utf8(data).expect("Making utf8 string failed")
    );
    Ok(())
}
