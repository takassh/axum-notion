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
pub struct TraceWithFullDetails {
    /// Path of trace in Langfuse UI
    #[serde(rename = "htmlPath")]
    pub html_path: String,
    /// Cost of trace in USD
    #[serde(rename = "totalCost")]
    pub total_cost: f64,
    #[serde(rename = "observations")]
    pub observations: Vec<models::ObservationsView>,
    #[serde(rename = "scores")]
    pub scores: Vec<models::Score>,
    /// The unique identifier of a trace
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "name", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub name: Option<Option<String>>,
    #[serde(rename = "input", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub input: Option<Option<serde_json::Value>>,
    #[serde(rename = "output", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub output: Option<Option<serde_json::Value>>,
    #[serde(rename = "sessionId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Option<String>>,
    #[serde(rename = "release", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub release: Option<Option<String>>,
    #[serde(rename = "version", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub version: Option<Option<String>>,
    #[serde(rename = "userId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Option<String>>,
    #[serde(rename = "metadata", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(rename = "tags", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Option<Vec<String>>>,
    /// Public traces are accessible via url without login
    #[serde(rename = "public", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub public: Option<Option<bool>>,
}

impl TraceWithFullDetails {
    pub fn new(html_path: String, total_cost: f64, observations: Vec<models::ObservationsView>, scores: Vec<models::Score>, id: String, timestamp: String) -> TraceWithFullDetails {
        TraceWithFullDetails {
            html_path,
            total_cost,
            observations,
            scores,
            id,
            timestamp,
            name: None,
            input: None,
            output: None,
            session_id: None,
            release: None,
            version: None,
            user_id: None,
            metadata: None,
            tags: None,
            public: None,
        }
    }
}

