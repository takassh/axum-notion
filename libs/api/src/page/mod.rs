use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use repositories::{RepositoriesError, Repository};

mod request;
pub mod response;

use self::response::{GetPageResponse, GetPagesResponse, Page};

fn into_response(e: RepositoriesError, message: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{message}: {e}"))
        .into_response()
}

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

// pub async fn delete_page(
//     State(repo): State<Repository>,
//     Json(id): Json<String>,
// ) -> Result<impl IntoResponse, Response> {
//     match repo.page.delete_by_id(id).await {
//         Ok(v) => Ok(v),
//         Err(e) => Err(into_response(e, "delete by id")),
//     }
// }
