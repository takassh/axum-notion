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
pub struct CreateTextPromptRequest {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "prompt")]
    pub prompt: String,
    #[serde(
        rename = "config",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub config: Option<Option<serde_json::Value>>,
    /// List of deployment labels of this prompt version.
    #[serde(
        rename = "labels",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub labels: Option<Option<Vec<String>>>,
    /// List of tags to apply to all versions of this prompt.
    #[serde(
        rename = "tags",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub tags: Option<Option<Vec<String>>>,
}

impl CreateTextPromptRequest {
    pub fn new(name: String, prompt: String) -> CreateTextPromptRequest {
        CreateTextPromptRequest {
            name,
            prompt,
            config: None,
            labels: None,
            tags: None,
        }
    }
}
