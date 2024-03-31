use axum::{
    extract::{Path, Query, State},
    Json,
};
use repository::Repository;

pub mod request;
pub mod response;

use crate::response::{ApiResponse, IntoApiResponse};

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
    State(repo): State<Repository>,
    Query(params): Query<GetPagesParam>,
) -> ApiResponse<Json<GetPagesResponse>> {
    let pages = repo
        .page
        .find_paginate(params.pagination.offset, params.pagination.limit)
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
    State(repo): State<Repository>,
    Path(id): Path<String>,
) -> ApiResponse<Json<GetPageResponse>> {
    let page = repo.page.find_by_id(id).await.into_response("502-002")?;

    let Some(page) = page else {
        return Ok(Json(GetPageResponse { page: None }));
    };

    Ok(Json(GetPageResponse {
        page: Some(Page {
            contents: page.contents,
        }),
    }))
}
