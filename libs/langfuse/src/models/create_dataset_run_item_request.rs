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
pub struct CreateDatasetRunItemRequest {
    #[serde(rename = "runName")]
    pub run_name: String,
    /// Description of the run. If run exists, description will be updated.
    #[serde(rename = "runDescription", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub run_description: Option<Option<String>>,
    /// Metadata of the dataset run, updates run if run already exists
    #[serde(rename = "metadata", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Option<serde_json::Value>>,
    #[serde(rename = "datasetItemId")]
    pub dataset_item_id: String,
    #[serde(rename = "observationId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub observation_id: Option<Option<String>>,
    /// traceId should always be provided. For compatibility with older SDK versions it can also be inferred from the provided observationId.
    #[serde(rename = "traceId", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Option<String>>,
}

impl CreateDatasetRunItemRequest {
    pub fn new(run_name: String, dataset_item_id: String) -> CreateDatasetRunItemRequest {
        CreateDatasetRunItemRequest {
            run_name,
            run_description: None,
            metadata: None,
            dataset_item_id,
            observation_id: None,
            trace_id: None,
        }
    }
}

