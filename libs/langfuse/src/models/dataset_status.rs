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
pub enum DatasetStatus {
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "ARCHIVED")]
    Archived,
}

impl std::fmt::Display for DatasetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}

impl Default for DatasetStatus {
    fn default() -> DatasetStatus {
        Self::Active
    }
}
