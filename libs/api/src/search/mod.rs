use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{PromptRequest, TextGeneration, TextGenerationRequest},
};
use qdrant_client::qdrant::SearchPoints;

use crate::{
    response::{ApiResponse, IntoApiResponse},
    ApiState,
};

use self::{request::SearchParam, response::SearchResponse};

pub mod request;
pub mod response;

/// Search with prompt
#[utoipa::path(
    get,
    path = "/search",
    responses(
        (status = 200, description = "Search with prompt successfully", body = [SearchResponse])
    ),
    params(
        SearchParam
    )
)]
pub async fn search_text(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<SearchParam>,
) -> ApiResponse<Json<SearchResponse>> {
    let embedding = state
        .cloudflare
        .bge_small_en_v1_5(TextEmbeddingsRequest {
            text: StringOrArray::from(params.prompt.as_str()),
        })
        .await
        .into_response("502-012")?;

    let Some(vector) = embedding.result.data.first() else {
        return Err(anyhow::anyhow!("No vectors found"))
            .into_response("502-012")?;
    };

    let search_result = state
        .qdrant
        .search_points(&SearchPoints {
            collection_name: state.config.qdrant.collection.clone(),
            vector: vector.clone(),
            limit: 10,
            ..Default::default()
        })
        .await
        .into_response("502-013")?;

    let mut context: Vec<&str> = vec![];
    for result in search_result.result.iter() {
        let Some(doc) = result.payload.get("document") else {
            continue;
        };
        let Some(doc) = doc.as_str() else {
            continue;
        };

        context.push(doc);
    }

    let prompt = format!(
        r#"
        You are an assistant helping a user to search for something.
        The user provides a prompt "{}" and you need to generate a response based on given contexts.
        Your answer must be concise.

        Context:
        "{}"
        
        Answer:
        "#,
        params.prompt,
        context.join("\n")
    );

    let response = state
        .cloudflare
        .llama_3_8b_instruct(TextGenerationRequest::Prompt(PromptRequest {
            prompt: prompt.to_string(),
            ..Default::default()
        }))
        .await
        .into_response("502-014")?;

    Ok(Json(SearchResponse {
        answer: response.result.response,
    }))
}
