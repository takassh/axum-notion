use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use entity::page::ParentType;

pub mod request;
pub mod response;

use crate::{
    response::{ApiResponse, IntoApiResponse},
    ws,
};
use crate::{
    ws::request::{GenerateImageFromText, GenerateImageRequest},
    ApiState,
};

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
    State(state): State<Arc<ApiState>>,
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
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<GenerateImageRequest>,
) -> ApiResponse<()> {
    ws::generate_cover_image(&state, id, body)
        .await
        .into_response("502-010")?;

    Ok(())
}

/// Generate a cover image from plain texts
#[utoipa::path(
    post,
    path = "/pages/:id/generate-cover-image-from-plain-texts",
    responses(
        (status = 200, description = "Generate a cover image for a page successfully", body = [GetPagesResponse])
    ),
    params(
        ("id", description = "page id"),
    )
)]
pub async fn generate_cover_image_from_plain_texts(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<GenerateImageFromText>,
) -> ApiResponse<()> {
    ws::generate_cover_image_from_plain_text(&state, id, body.text)
        .await
        .into_response("502-010")?;

    Ok(())
}

/// Generate a cover image from plain texts
#[utoipa::path(
    post,
    path = "/pages/:id/summarize",
    responses(
        (status = 200, description = "Generate a cover image for a page successfully", body = [GetPagesResponse])
    ),
    params(
        ("id", description = "page id"),
    )
)]
pub async fn generate_summarize(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<GenerateImageFromText>,
) -> ApiResponse<()> {
    ws::summarize(&state, id, body.text)
        .await
        .into_response("502-011")?;

    Ok(())
}
