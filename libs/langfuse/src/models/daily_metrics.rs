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
pub struct DailyMetrics {
    /// A list of daily metrics, only days with ingested data are included.
    #[serde(rename = "data")]
    pub data: Vec<models::DailyMetricsDetails>,
    #[serde(rename = "meta")]
    pub meta: Box<models::UtilsMetaResponse>,
}

impl DailyMetrics {
    pub fn new(
        data: Vec<models::DailyMetricsDetails>,
        meta: models::UtilsMetaResponse,
    ) -> DailyMetrics {
        DailyMetrics {
            data,
            meta: Box::new(meta),
        }
    }
}
