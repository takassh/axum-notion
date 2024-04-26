use axum::{
    extract::{Path, Query, State},
    Json,
};

pub mod request;
pub mod response;

use crate::ApiState;
use crate::{
    response::{ApiResponse, IntoApiResponse},
    ws,
};

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
    ws::generate_cover_image(&state, id, body)
        .await
        .into_response("502-010")?;

    Ok(())
}
