use std::time::Duration;

use anyhow::*;
use graphql_client::*;
use log::*;
// use prettytable::*;
use serde::*;
use structopt::StructOpt;
// use chrono::Utc;
use chrono::Local;
use csv::WriterBuilder;

type URI = String;
type DateTime = chrono::DateTime<Local>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "./data/schema.graphql",
    query_path = "./data/issues.graphql",
    response_derives = "Debug"
)]
struct ListIssuesQuery;

#[derive(Serialize, Debug)]
struct IssueRow<'x> {
    pub id: &'x str,
    pub number: i64,
    pub title: &'x str,
    pub url: &'x str,
    pub assignees: &'x str,
    pub active_lock_reason: Option<&'x list_issues_query::LockReason>,
    pub author_email: &'x str,
    pub author_login: &'x str,
    pub author_association: i32,
    pub body: &'x str,
    pub closed: bool,
    pub closed_at: Option<DateTime>,
    pub created_at: DateTime,
    pub comments_count: i32,
    pub created_via_email: bool,
    pub database_id: Option<i64>,
    pub editor_login: &'x str,
    pub includes_created_edit: bool,
    pub labels: &'x str,
    pub last_edited_at: Option<DateTime>,
    pub locked: bool,
    pub milestone_title: &'x str,
    pub milestone_number: i32,
    pub published_at: Option<DateTime>,
    pub resource_path: String,
    pub state: &'x list_issues_query::IssueState,
    pub updated_at: DateTime,
    // participants(first: 100) {
    //   totalCount
    //   nodes {
    //     login
    //     name
    //   }
    // }
}

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
    let mut has_header = true;
    let mut first = true;
    let mut cursor = String::from("");

    while has_next {
        let after: Option<String> = if first { None } else { Some(cursor.clone()) };
        let q = ListIssuesQuery::build_query(list_issues_query::Variables {
            name: name.clone(),
            owner: owner.clone(),
            after: after,
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
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
        let mut wtr = WriterBuilder::new()
            .has_headers(has_header)
            .from_writer(vec![]);
        for issue in nodes {
            let issue = issue.as_ref().unwrap();
            let author = issue.author.as_ref().unwrap();
            if let User(author) = &author.on {
                let row = IssueRow {
                    id: &issue.id,
                    number: issue.number,
                    title: &issue.title,
                    url: &issue.url,
                    assignees: "",
                    active_lock_reason: issue.active_lock_reason.as_ref(),
                    author_email: &author.email,
                    author_login: &author.login,
                    author_association: 0,
                    body: &issue.body,
                    closed: issue.closed,
                    closed_at: issue.closed_at,
                    created_at: issue.created_at,
                    comments_count: 0,
                    created_via_email: issue.created_via_email,
                    database_id: issue.database_id,
                    editor_login: "",
                    includes_created_edit: false,
                    labels: "",
                    last_edited_at: issue.last_edited_at,
                    locked: issue.locked,
                    milestone_title: "",
                    milestone_number: 1,
                    published_at: issue.published_at,
                    resource_path: issue.resource_path.clone(),
                    state: &issue.state,
                    updated_at: issue.updated_at,
                };
                wtr.serialize(&row)?;
            }
        }
        println!("{}", String::from_utf8(wtr.into_inner()?)?);
        eprintln!("Iterated");
        has_header = false; // Force false in 2nd loop
    }

    Ok(())
}
