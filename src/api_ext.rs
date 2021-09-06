pub mod models {
    use octocrab::models::workflows::HeadCommit;
    use octocrab::models::events::Repository;
    use reqwest::Url;
    use serde::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct RepositoryMinimal {
        id: i64,
        url: Url,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct HeadMininal {
        #[serde(rename = "ref")]
        pub ref_field: String,
        pub sha: String,
        pub repo: RepositoryMinimal,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct BaseMininal {
        #[serde(rename = "ref")]
        pub ref_field: String,
        pub sha: String,
        pub repo: RepositoryMinimal,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct PullReuqestMinimal {
        pub id: i64,
        pub number: i64,
        pub url: Url,
        pub head: HeadMininal,
        pub base: BaseMininal,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct Run {
        pub id: i64,
        pub workflow_id: i64,
        pub node_id: String,
        pub name: String,
        pub head_branch: String,
        pub head_sha: String,
        pub run_number: i64,
        pub event: String,
        pub status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub conclusion: Option<String>,
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
        pub url: Url,
        pub html_url: Url,
        pub jobs_url: Url,
        pub logs_url: Url,
        pub check_suite_url: Url,
        pub artifacts_url: Url,
        pub cancel_url: Url,
        pub rerun_url: Url,
        pub workflow_url: Url,
        pub pull_requests: Vec<PullReuqestMinimal>,
        // TODO: other attrs
        // ref: https://docs.github.com/en/rest/reference/actions#list-workflow-runs
        pub head_commit: HeadCommit,
        pub repository: Repository,
        pub head_repository: Repository,
    }
}