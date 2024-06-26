/*
 * langfuse
 *
 * ## Authentication  Authenticate with the API using [Basic Auth](https://en.wikipedia.org/wiki/Basic_access_authentication), get API keys in the project settings:  - username: Langfuse Public Key - password: Langfuse Secret Key  ## Exports  - OpenAPI spec: https://cloud.langfuse.com/generated/api/openapi.yml - Postman collection: https://cloud.langfuse.com/generated/postman/collection.json
 *
 * The version of the OpenAPI document:
 *
 * Generated by: https://openapi-generator.tech
 */

use super::{configuration, Error};
use crate::{apis::ResponseContent, models};
use reqwest;
use serde::{Deserialize, Serialize};

/// struct for typed errors of method [`datasets_create`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatasetsCreateError {
    Status400(serde_json::Value),
    Status401(serde_json::Value),
    Status403(serde_json::Value),
    Status404(serde_json::Value),
    Status405(serde_json::Value),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`datasets_get`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatasetsGetError {
    Status400(serde_json::Value),
    Status401(serde_json::Value),
    Status403(serde_json::Value),
    Status404(serde_json::Value),
    Status405(serde_json::Value),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`datasets_get_runs`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatasetsGetRunsError {
    Status400(serde_json::Value),
    Status401(serde_json::Value),
    Status403(serde_json::Value),
    Status404(serde_json::Value),
    Status405(serde_json::Value),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`datasets_list`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatasetsListError {
    Status400(serde_json::Value),
    Status401(serde_json::Value),
    Status403(serde_json::Value),
    Status404(serde_json::Value),
    Status405(serde_json::Value),
    UnknownValue(serde_json::Value),
}

/// Create a dataset
pub async fn datasets_create(
    configuration: &configuration::Configuration,
    create_dataset_request: models::CreateDatasetRequest,
) -> Result<models::Dataset, Error<DatasetsCreateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str =
        format!("{}/api/public/datasets", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client
        .request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder
            .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_auth_conf) = local_var_configuration.basic_auth {
        local_var_req_builder = local_var_req_builder.basic_auth(
            local_var_auth_conf.0.to_owned(),
            local_var_auth_conf.1.to_owned(),
        );
    };
    local_var_req_builder = local_var_req_builder.json(&create_dataset_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error()
        && !local_var_status.is_server_error()
    {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DatasetsCreateError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get a dataset and its items
pub async fn datasets_get(
    configuration: &configuration::Configuration,
    dataset_name: &str,
) -> Result<models::Dataset, Error<DatasetsGetError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/public/datasets/{datasetName}",
        local_var_configuration.base_path,
        datasetName = crate::apis::urlencode(dataset_name)
    );
    let mut local_var_req_builder = local_var_client
        .request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder
            .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_auth_conf) = local_var_configuration.basic_auth {
        local_var_req_builder = local_var_req_builder.basic_auth(
            local_var_auth_conf.0.to_owned(),
            local_var_auth_conf.1.to_owned(),
        );
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error()
        && !local_var_status.is_server_error()
    {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DatasetsGetError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get a dataset run and its items
pub async fn datasets_get_runs(
    configuration: &configuration::Configuration,
    dataset_name: &str,
    run_name: &str,
) -> Result<models::DatasetRun, Error<DatasetsGetRunsError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!(
        "{}/api/public/datasets/{datasetName}/runs/{runName}",
        local_var_configuration.base_path,
        datasetName = crate::apis::urlencode(dataset_name),
        runName = crate::apis::urlencode(run_name)
    );
    let mut local_var_req_builder = local_var_client
        .request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder
            .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_auth_conf) = local_var_configuration.basic_auth {
        local_var_req_builder = local_var_req_builder.basic_auth(
            local_var_auth_conf.0.to_owned(),
            local_var_auth_conf.1.to_owned(),
        );
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error()
        && !local_var_status.is_server_error()
    {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DatasetsGetRunsError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get all datasets
pub async fn datasets_list(
    configuration: &configuration::Configuration,
    page: Option<i32>,
    limit: Option<i32>,
) -> Result<models::PaginatedDatasets, Error<DatasetsListError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str =
        format!("{}/api/public/datasets", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client
        .request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = page {
        local_var_req_builder = local_var_req_builder
            .query(&[("page", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = limit {
        local_var_req_builder = local_var_req_builder
            .query(&[("limit", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder
            .header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_auth_conf) = local_var_configuration.basic_auth {
        local_var_req_builder = local_var_req_builder.basic_auth(
            local_var_auth_conf.0.to_owned(),
            local_var_auth_conf.1.to_owned(),
        );
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error()
        && !local_var_status.is_server_error()
    {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<DatasetsListError> =
            serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent {
            status: local_var_status,
            content: local_var_content,
            entity: local_var_entity,
        };
        Err(Error::ResponseError(local_var_error))
    }
}
