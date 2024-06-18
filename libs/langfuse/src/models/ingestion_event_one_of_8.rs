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
pub struct IngestionEventOneOf8 {
    #[serde(rename = "body")]
    pub body: Box<models::ObservationBody>,
    /// UUID v4 that identifies the event
    #[serde(rename = "id")]
    pub id: String,
    /// Datetime (ISO 8601) of event creation in client. Should be as close to actual event creation in client as possible, this timestamp will be used for ordering of events in future release. Resolution: milliseconds (required), microseconds (optimal).
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    /// Optional. Metadata field used by the Langfuse SDKs for debugging.
    #[serde(
        rename = "metadata",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(rename = "type")]
    pub r#type: Type,
}

impl IngestionEventOneOf8 {
    pub fn new(
        body: models::ObservationBody,
        id: String,
        timestamp: String,
        r#type: Type,
    ) -> IngestionEventOneOf8 {
        IngestionEventOneOf8 {
            body: Box::new(body),
            id,
            timestamp,
            metadata: None,
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
    #[serde(rename = "observation-create")]
    ObservationCreate,
}

impl Default for Type {
    fn default() -> Type {
        Self::ObservationCreate
    }
}
