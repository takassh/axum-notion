/*
 * langfuse
 *
 * ## Authentication  Authenticate with the API using [Basic Auth](https://en.wikipedia.org/wiki/Basic_access_authentication), get API keys in the project settings:  - username: Langfuse Public Key - password: Langfuse Secret Key  ## Exports  - OpenAPI spec: https://cloud.langfuse.com/generated/api/openapi.yml - Postman collection: https://cloud.langfuse.com/generated/postman/collection.json
 *
 * The version of the OpenAPI document:
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromptOneOf {
    #[serde(rename = "prompt")]
    pub prompt: Vec<models::ChatMessage>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "version")]
    pub version: i32,
    #[serde(rename = "config", deserialize_with = "Option::deserialize")]
    pub config: Option<serde_json::Value>,
    /// List of deployment labels of this prompt version.
    #[serde(rename = "labels")]
    pub labels: Vec<String>,
    /// List of tags. Used to filter via UI and API. The same across versions of a prompt.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
    #[serde(rename = "type")]
    pub r#type: Type,
}

impl PromptOneOf {
    pub fn new(
        prompt: Vec<models::ChatMessage>,
        name: String,
        version: i32,
        config: Option<serde_json::Value>,
        labels: Vec<String>,
        tags: Vec<String>,
        r#type: Type,
    ) -> PromptOneOf {
        PromptOneOf {
            prompt,
            name,
            version,
            config,
            labels,
            tags,
            r#type,
        }
    }
}
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum Type {
    #[serde(rename = "chat")]
    Chat,
}

impl Default for Type {
    fn default() -> Type {
        Self::Chat
    }
}
