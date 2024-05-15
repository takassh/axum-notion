use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use cloudflare::models::text_generation::Message;
use cloudflare::models::text_generation::MessageRequest;
use cloudflare::models::text_generation::TextGeneration;
use cloudflare::models::text_generation::TextGenerationRequest;
use cloudflare::models::text_to_image::{TextToImage, TextToImageRequest};
use entity::page::ParentType;
use entity::prelude::PageEntity;
use notion_client::objects::page::PageProperty;
use notion_client::objects::rich_text::RichText;
use notion_client::objects::rich_text::Text;
use notion_client::{
    endpoints::pages::update::request::UpdatePagePropertiesRequest,
    objects::{
        file::{ExternalFile, File},
        parent::Parent,
    },
};

pub mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};
use crate::ApiState;

use self::request::GenerateCoverImageParam;
use self::request::GenerateSummarizeParam;
use self::{
    request::GetPagesParam,
    response::{GetPageResponse, GetPagesResponse, ResponsePage},
};

/// List all pages
#[utoipa::path(
        get,
        path = "/pages",
        responses(
            (status = 200, description = "List all pages successfully", body = [GetPagesResponse])
        ),
        params(
            GetPagesParam
        )
    )]
pub async fn get_pages(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<GetPagesParam>,
) -> ApiResponse<Json<GetPagesResponse>> {
    let pages = state
        .repo
        .page
        .find_paginate(
            params.pagination.page,
            params.pagination.limit,
            params.category,
            Some(ParentType::Database),
        )
        .await
        .into_response("502-001")?;

    let response = Json(GetPagesResponse {
        pages: pages
            .into_iter()
            .map(|a| ResponsePage {
                contents: a.contents,
            })
            .collect(),
    });

    Ok(response)
}

/// List a page
#[utoipa::path(
    get,
    path = "/pages/:id",
    responses(
        (status = 200, description = "List all page items successfully", body = [GetPageResponse])
    ),
    params(
        ("id", description = "page id"),
    )
)]
pub async fn get_page(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> ApiResponse<Json<GetPageResponse>> {
    let page = state
        .repo
        .page
        .find_by_id(&id)
        .await
        .into_response("502-002")?;

    let Some(page) = page else {
        return Ok(Json(GetPageResponse { page: None }));
    };

    Ok(Json(GetPageResponse {
        page: Some(ResponsePage {
            contents: page.contents,
        }),
    }))
}

/// Generate a cover image for a page
#[utoipa::path(
    post,
    path = "/pages/:id/generate-cover-image",
    responses(
        (status = 200, description = "Generate a cover image for a page successfully", body = [GetPagesResponse])
    ),
    params(
        ("id", description = "page id"),
    )
)]
pub async fn generate_cover_image(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<GenerateCoverImageParam>,
) -> ApiResponse<()> {
    let bytes = state
        .cloudflare
        .stable_diffusion_xl_lightning(TextToImageRequest {
            prompt: body.prompt,
            ..Default::default()
        })
        .await
        .into_response("502-009")?;

    let file_name = format!("{}.png", id);

    state
        .s3
        .put_object()
        .bucket(state.config.aws.bucket.clone())
        .content_type("image/png")
        .key(file_name.clone())
        .body(ByteStream::from(bytes))
        .send()
        .await
        .context("failed to upload image to s3")
        .into_response("502-016")?;

    state
        .notion
        .pages
        .update_page_properties(
            &id,
            UpdatePagePropertiesRequest {
                cover: Some(File::External {
                    external: ExternalFile {
                        url: format!(
                            "{}/{}?t={}",
                            state.config.aws.s3_url,
                            file_name,
                            chrono::Utc::now().timestamp()
                        ),
                    },
                }),
                ..Default::default()
            },
        )
        .await
        .context("failed to update page properties")
        .into_response("502-017")?;

    save_page(&state, &id).await.into_response("502-018")
}

/// Summerize a page content
#[utoipa::path(
    post,
    path = "/pages/:id/summarize",
    responses(
        (status = 200, description = "Summerize a page content successfully", body = [GetPagesResponse])
    ),
    params(
        ("id", description = "page id"),
    )
)]
pub async fn generate_summarize(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<GenerateSummarizeParam>,
) -> ApiResponse<()> {
    let response =state.cloudflare.llama_3_8b_instruct(TextGenerationRequest::Message(MessageRequest { messages:  vec![
        Message {
            role: "system".to_string(),
            content: "You will summarize texts. You must reply only the summary. You can't add any other contexts.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content:  format!(r#"
            Please summarize the following text.
            "{}"
            summary:
            "#,body.text).to_string(),
        },
    ], ..Default::default()})).await.into_response("502-011")?;

    let mut properties = BTreeMap::new();
    properties.insert(
        "summary".to_string(),
        PageProperty::RichText {
            id: None,
            rich_text: vec![RichText::Text {
                text: Text {
                    content: response.result.response,
                    link: None,
                },
                annotations: None,
                plain_text: None,
                href: None,
            }],
        },
    );

    state
        .notion
        .pages
        .update_page_properties(
            &id,
            UpdatePagePropertiesRequest {
                properties,
                ..Default::default()
            },
        )
        .await
        .context("failed to update page properties")
        .into_response("502-017")?;

    save_page(&state, &id).await.into_response("502-018")
}

async fn save_page(state: &ApiState, id: &str) -> anyhow::Result<()> {
    let page = state
        .notion
        .pages
        .retrieve_a_page(id, None)
        .await
        .context("failed to retrieve a page")?;

    let json = serde_json::to_string_pretty(&page)
        .context("failed to serialize page")?;
    let parent_id = match page.parent {
        Parent::DatabaseId { database_id } => database_id,
        _ => Err(anyhow!("parent is not database id"))?,
    };
    let model = PageEntity {
        notion_page_id: page.id,
        notion_parent_id: parent_id,
        contents: json,
        created_at: page.created_time,
        ..Default::default()
    };

    state
        .repo
        .page
        .save(model)
        .await
        .context("failed to save page")?;

    Ok(())
}
