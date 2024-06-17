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
pub struct DatasetWithReferences {
    /// list of dataset item ids
    #[serde(rename = "items")]
    pub items: Vec<String>,
    /// list of dataset run names
    #[serde(rename = "runs")]
    pub runs: Vec<String>,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "description", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    #[serde(rename = "metadata", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl DatasetWithReferences {
    pub fn new(items: Vec<String>, runs: Vec<String>, id: String, name: String, project_id: String, created_at: String, updated_at: String) -> DatasetWithReferences {
        DatasetWithReferences {
            items,
            runs,
            id,
            name,
            description: None,
            metadata: None,
            project_id,
            created_at,
            updated_at,
        }
    }
}

