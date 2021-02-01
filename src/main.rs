use anyhow::*;
use graphql_client::*;
use log::*;
// use prettytable::*;
use serde::*;
use structopt::StructOpt;
// use chrono::Utc;
use chrono::Local;

type URI = String;
type DateTime = chrono::DateTime<Local>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "./data/schema.graphql",
    query_path = "./data/issues.graphql",
    response_derives = "Debug"
)]
struct ListIssuesQuery;

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

use list_issues_query::ListIssuesQueryRepositoryIssuesNodesAuthorOn::User;

fn main() -> Result<()> {
    env_logger::init();

    let config: Env = envy::from_env().context("while reading from environment")?;
    let args = Command::from_args();
    let owner = args.owner;
    let name = args.name;
    let mut has_next = true;
    let mut first = true;
    let mut cursor = String::from("");

    while has_next {
        let after: Option<String> = if first {
            None
        } else {
            Some(cursor.clone())
        };
        let q = ListIssuesQuery::build_query(list_issues_query::Variables {
            owner: owner.clone(),
            name: name.clone(),
            after: after,
        });

        let client = reqwest::blocking::Client::builder()
            .user_agent("ghex(graphql-rust/0.9.0)")
            .build()?;

        let res = client
            .post(&config.github_api_url)
            .bearer_auth(config.github_api_token.clone())
            .json(&q)
            .send()?;

        res.error_for_status_ref()?;

        let response_body: Response<list_issues_query::ResponseData> = res.json()?;

        if let Some(errors) = response_body.errors {
            error!("there are errors:");

            for error in &errors {
                error!("{:?}", error);
            }
            bail!("Request failed")
        }

        let response_data = response_body.data.expect("missing response data");
        debug!("{:?}", response_data);
        let issues = &response_data
            .repository
            .as_ref()
            .expect("No Repo Found")
            .issues;

        has_next = issues.page_info.has_next_page;
        if has_next {
            first = false;
            let cursor_ = issues.page_info.end_cursor.as_ref();
            cursor = String::from(cursor_.unwrap());
        }
        let nodes = issues.nodes.as_ref().expect("No Issues");
        for issue in nodes {
            let issue = issue.as_ref().unwrap();
            let author = issue.author.as_ref().unwrap();
            if let User(author) = &author.on {
                println!("{},{},{},{},{},{}",
                         issue.number,
                         issue.title,
                         author.email,
                         author.login,
                         issue.created_at,
                         issue.updated_at
                );
            }
        }
    }

    Ok(())
}
