use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Event {
    pub id: String,
    pub r#type: Option<String>,
    pub actor: Actor,
    pub repo: Repo,
    pub org: Option<Actor>,
    pub payload: Payload,
    pub public: bool,
    pub created_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Actor {
    pub id: i64,
    pub login: String,
    pub display_login: Option<String>,
    pub gravatar_id: Option<String>,
    pub url: String,
    pub avatar_url: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub url: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Payload {
    pub action: Option<String>,
    pub issue: Option<Issue>,
    pub comment: Option<Comment>,
    pub pages: Option<Vec<Page>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Issue {
    pub id: i64,
    pub node_id: String,
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: i64,
    pub state: String,
    pub state_reason: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub user: Option<SimpleUser>,
    pub labels: Vec<LabelOrString>,
    pub assignee: Option<SimpleUser>,
    pub assignees: Option<Vec<SimpleUser>>,
    pub milestone: Option<Milestone>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum LabelOrString {
    Label(Label),
    String(String),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Label {
    pub id: i64,
    pub node_id: String,
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub default: bool,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SimpleUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: Option<String>,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    pub r#type: String,
    pub site_admin: bool,
    pub starred_at: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Milestone {
    pub url: String,
    pub html_url: String,
    pub labels_url: String,
    pub id: i64,
    pub node_id: String,
    pub number: i64,
    pub state: String,
    pub title: String,
    pub description: Option<String>,
    pub creator: Option<SimpleUser>,
    pub open_issues: i64,
    pub closed_issues: i64,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub due_on: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Comment {
    pub id: i64,
    pub node_id: String,
    pub url: String,
    pub body: String,
    pub body_text: String,
    pub body_html: String,
    pub html_url: String,
    pub user: Option<SimpleUser>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub issue_url: String,
    pub author_association: String,
    pub performed_via_github_app: Option<GitHubApp>,
    pub reactions: ReactionRollup,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GitHubApp {
    pub id: i64,
    pub slug: String,
    pub node_id: String,
    pub owner: Option<SimpleUser>,
    pub name: String,
    pub description: Option<String>,
    pub external_url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub permissions: Permissions,
    pub events: Vec<String>,
    pub installations_count: i64,
    pub client_id: String,
    pub client_secret: String,
    pub webhook_secret: Option<String>,
    pub pem: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Permissions {
    pub issues: String,
    pub checks: String,
    pub metadata: String,
    pub contents: String,
    pub deployments: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ReactionRollup {
    pub url: String,
    pub total_count: i64,
    #[serde(rename = "+1")]
    pub plus_one: i64,
    #[serde(rename = "-1")]
    pub minus_one: i64,
    pub laugh: i64,
    pub confused: i64,
    pub heart: i64,
    pub hooray: i64,
    pub eyes: i64,
    pub rocket: i64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Page {
    pub page_name: String,
    pub title: String,
    pub summary: Option<String>,
    pub action: String,
    pub sha: String,
    pub html_url: String,
}
