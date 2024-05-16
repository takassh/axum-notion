use async_stream::stream;
use axum::{
    extract::{Query, State},
    response::{sse::Event, Sse},
    Json,
};
use cloudflare::models::{
    text_embeddings::{StringOrArray, TextEmbeddings, TextEmbeddingsRequest},
    text_generation::{PromptRequest, TextGeneration, TextGenerationRequest},
};
use futures_util::{pin_mut, Stream};
use qdrant_client::qdrant::{
    with_payload_selector::SelectorOptions, PayloadIncludeSelector,
    SearchPoints, WithPayloadSelector,
};
use std::{convert::Infallible, sync::Arc};
use tokio_stream::StreamExt as _;
use tracing::error;

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
    let context = retriever(&state, &params.prompt)
        .await
        .into_response("502-012")?;

    let prompt = format!(
        r#"
        You are an assistant helping a user to search for something.
        The user provides a prompt "{}" and you need to generate a response based on given contexts.
        If the context doesn't make sense with the prompt, you should answer you don't know.
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

pub async fn search_text_with_sse(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<SearchParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
    let context = retriever(&state, &params.prompt).await;
    let Ok(context) = context else {
        error!(
            task = "get context by retriever",
            error = context.unwrap_err().to_string(),
        );
        return;
    };

    let prompt = format!(
        r#"
        You are an assistant helping a user to search for something.
        The user provides a prompt "{}" and you need to generate a response based on given contexts.
        If the context doesn't make sense with the prompt, you should answer you don't know.
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
        .llama_3_8b_instruct_with_stream(TextGenerationRequest::Prompt(
            PromptRequest {
                prompt: prompt.to_string(),
                stream: Some(true),
                ..Default::default()
            },
        ));


        pin_mut!(response); // needed for iteration

        // receive response from LLM
        while let Ok(Some(data)) = response.next().await.transpose() {
            for d in data{
                let event =  Event::default().json_data(d);
                match event {
                    Ok(event) => yield event,
                    Err(err) => error!(
                        task = "event json_data",
                        error = err.to_string()
                    ),
                }
            }
        }
    };

    Sse::new(stream.map(Ok))
}

async fn retriever(
    state: &Arc<ApiState>,
    prompt: &str,
) -> anyhow::Result<Vec<String>> {
    let embedding = state
        .cloudflare
        .bge_small_en_v1_5(TextEmbeddingsRequest {
            text: StringOrArray::from(prompt),
        })
        .await?;

    let Some(vector) = embedding.result.data.first() else {
        return Err(anyhow::anyhow!("No vectors found"));
    };

    let search_result = state
        .qdrant
        .search_points(&SearchPoints {
            collection_name: state.config.qdrant.collection.clone(),
            vector: vector.clone(),
            limit: 3,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(SelectorOptions::Include(
                    PayloadIncludeSelector {
                        fields: vec!["document".to_string()],
                    },
                )),
            }),
            ..Default::default()
        })
        .await?;

    let mut context: Vec<String> = vec![];
    for result in search_result.result.iter() {
        if result.score < 0.6 {
            continue;
        }
        let Some(doc) = result.payload.get("document") else {
            continue;
        };
        let Some(doc) = doc.as_str() else {
            continue;
        };

        context.push(doc.to_string());
    }

    Ok(context)
}
