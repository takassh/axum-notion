use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use repositories::Repository;
mod request;
mod response;

use self::response::{Block, GetBlockRespose, GetBlocksRespose};

type ResponseError = (StatusCode, String);

pub async fn get_blocks(
    State(repo): State<Repository>,
) -> Result<Json<GetBlocksRespose>, ResponseError> {
    let blocks = repo.block.find_all().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to get blocks: {e}"),
        )
    })?;

    let response = Json(GetBlocksRespose {
        blocks: blocks
            .into_iter()
            .map(|a| Block {
                parent_id: a.notion_page_id,
                contents: a.contents,
            })
            .collect(),
    });

    Ok(response)
}

pub async fn get_block(
    State(repo): State<Repository>,
    Path(id): Path<String>,
) -> Result<Json<GetBlockRespose>, ResponseError> {
    let block = repo.block.find_by_notion_page_id(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {e}"),
        )
    })?;

    let Some(block) = block else {
        return Ok(Json(GetBlockRespose { block: None }));
    };

    Ok(Json(GetBlockRespose {
        block: Some(Block {
            parent_id: block.notion_page_id,
            contents: block.contents,
        }),
    }))
}

// pub async fn delete_block(
//     State(repo): State<Repository>,
//     Json(id): Json<String>,
// ) -> Result<impl IntoResponse, ResponseError> {
//     match repo.block.delete_by_id(id).await {
//         Ok(v) => Ok(v),
//         Err(e) => Err((
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Something went wrong: {e}"),
//         )),
//     }
// }
