use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use repositories::Repository;

mod request;
pub mod response;

use crate::util::into_response;

use self::response::{GetPageResponse, GetPagesResponse, Page};

pub async fn get_pages(
    State(repo): State<Repository>,
) -> Result<Json<GetPagesResponse>, Response> {
    let pages = repo
        .page
        .find_all()
        .await
        .map_err(|e| into_response(e, "find all"))?;

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
) -> Result<Json<GetPageResponse>, Response> {
    let page = repo
        .page
        .find_by_id(id)
        .await
        .map_err(|e| into_response(e, "find by id"))?;

    let Some(page) = page else {
        return Ok(Json(GetPageResponse { page: None }));
    };

    Ok(Json(GetPageResponse {
        page: Some(Page {
            contents: page.contents,
        }),
    }))
}
