use anyhow::{anyhow, Context};

use aws_sdk_s3::primitives::ByteStream;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use entity::prelude::*;
use notion_client::objects::file::ExternalFile;
use notion_client::objects::file::File;
use notion_client::{
    endpoints::pages::update::request::UpdatePagePropertiesRequest,
    objects::parent::Parent,
};

pub mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};
use crate::ApiState;

use self::request::GenerateCoverImageRequest;
use self::{
    request::GetPagesParam,
    response::{GetPageResponse, GetPagesResponse, Page},
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
    State(state): State<ApiState>,
    Query(params): Query<GetPagesParam>,
) -> ApiResponse<Json<GetPagesResponse>> {
    let pages = state
        .repo
        .page
        .find_paginate(
            params.pagination.page,
            params.pagination.limit,
            params.category,
        )
        .await
        .into_response("502-001")?;

    let response = Json(GetPagesResponse {
        pages: pages
            .into_iter()
            .map(|a| Page {
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
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> ApiResponse<Json<GetPageResponse>> {
    let page = state
        .repo
        .page
        .find_by_id(id)
        .await
        .into_response("502-002")?;

    let Some(page) = page else {
        return Ok(Json(GetPageResponse { page: None }));
    };

    Ok(Json(GetPageResponse {
        page: Some(Page {
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
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(body): Json<GenerateCoverImageRequest>,
) -> ApiResponse<()> {
    let response = state
        .cloudflare
        .post(
            state.config.cloudflare.generate_ai_path.as_str(),
            serde_json::to_string(&body)
                .context("failed to serialize body")
                .into_response("502-009")?,
        )
        .await
        .into_response("502-009")?;

    let file_name = format!("{}.png", id);

    let image = response
        .bytes()
        .await
        .context("failed to get response bytes")
        .into_response("502-009")?;

    state
        .s3
        .put_object()
        .bucket(state.config.aws.bucket)
        .content_type("image/png")
        .key(file_name.clone())
        .body(ByteStream::from(image))
        .send()
        .await
        .context("failed to put object")
        .into_response("502-009")?;

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
        .into_response("502-009")?;

    let page = state
        .notion
        .pages
        .retrieve_a_page(&id, None)
        .await
        .context("failed to retrieve a page")
        .into_response("502-010")?;

    let json = serde_json::to_string_pretty(&page)
        .context("failed to serialize page")
        .into_response("502-010")?;
    let parent_id = match page.parent {
        Parent::DatabaseId { database_id } => database_id,
        _ => Err(anyhow!("parent is not database id"))
            .into_response("502-010")?,
    };
    let model = PageEntity {
        notion_page_id: page.id,
        notion_database_id: parent_id,
        contents: json,
        created_at: page.created_time,
        ..Default::default()
    };

    state
        .repo
        .page
        .save(model)
        .await
        .context("failed to save page")
        .into_response("502-010")?;

    Ok(())
}
