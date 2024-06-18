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
pub struct IngestionBatchRequest {
    /// Batch of tracing events to be ingested. Discriminated by attribute `type`.
    #[serde(rename = "batch")]
    pub batch: Vec<models::IngestionEvent>,
    /// Optional. Metadata field used by the Langfuse SDKs for debugging.
    #[serde(
        rename = "metadata",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata: Option<Option<serde_json::Value>>,
}

impl IngestionBatchRequest {
    pub fn new(batch: Vec<models::IngestionEvent>) -> IngestionBatchRequest {
        IngestionBatchRequest {
            batch,
            metadata: None,
        }
    }
}