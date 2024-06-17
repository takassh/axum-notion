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
pub struct UpdateSpanBody {
    #[serde(rename = "endTime", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Option<String>>,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "traceId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Option<String>>,
    #[serde(rename = "name", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub name: Option<Option<String>>,
    #[serde(rename = "startTime", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<Option<String>>,
    #[serde(rename = "metadata", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(rename = "input", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub input: Option<Option<serde_json::Value>>,
    #[serde(rename = "output", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub output: Option<Option<serde_json::Value>>,
    #[serde(rename = "level", skip_serializing_if = "Option::is_none")]
    pub level: Option<models::ObservationLevel>,
    #[serde(rename = "statusMessage", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<Option<String>>,
    #[serde(rename = "parentObservationId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub parent_observation_id: Option<Option<String>>,
    #[serde(rename = "version", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub version: Option<Option<String>>,
}

impl UpdateSpanBody {
    pub fn new(id: String) -> UpdateSpanBody {
        UpdateSpanBody {
            end_time: None,
            id,
            trace_id: None,
            name: None,
            start_time: None,
            metadata: None,
            input: None,
            output: None,
            level: None,
            status_message: None,
            parent_observation_id: None,
            version: None,
        }
    }
}

