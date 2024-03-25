use axum::{
    extract::{Path, State},
    Json,
};
use repositories::Repository;

mod request;
pub mod response;

use crate::{ApiResponse, IntoApiResponse};

use self::response::{GetPageResponse, GetPagesResponse, Page};

pub async fn get_pages(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetPagesResponse>> {
    let pages = repo.page.find_all().await.into_response("502-001")?;

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
