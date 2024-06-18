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
pub enum ScoreSource {
    #[serde(rename = "ANNOTATION")]
    Annotation,
    #[serde(rename = "API")]
    Api,
    #[serde(rename = "EVAL")]
    Eval,
}

impl std::fmt::Display for ScoreSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Annotation => write!(f, "ANNOTATION"),
            Self::Api => write!(f, "API"),
            Self::Eval => write!(f, "EVAL"),
        }
    }
}

impl Default for ScoreSource {
    fn default() -> ScoreSource {
        Self::Annotation
    }
}
