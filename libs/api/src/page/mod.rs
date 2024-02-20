use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use models::Repository;
mod request;
mod response;

use self::response::{GetPageRespose, GetPagesRespose, Page};

type ResponseError = (StatusCode, String);

pub async fn get_pages(
    State(repo): State<Repository>,
) -> Result<Json<GetPagesRespose>, ResponseError> {
    let pages = repo.page.find_all().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get pages: {e}"),
        )
    })?;

    let response = Json(GetPagesRespose {
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
) -> Result<Json<GetPageRespose>, ResponseError> {
    let page = repo.page.find_by_id(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {e}"),
        )
    })?;

    let Some(page) = page else {
        return Ok(Json(GetPageRespose { page: None }));
    };

    Ok(Json(GetPageRespose {
        page: Some(Page {
            contents: page.contents,
        }),
    }))
}

pub async fn delete_page(
    State(repo): State<Repository>,
    Json(id): Json<String>,
) -> Result<impl IntoResponse, ResponseError> {
    match repo.page.delete_by_id(id).await {
        Ok(v) => Ok(v),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {e}"),
        )),
    }
}
