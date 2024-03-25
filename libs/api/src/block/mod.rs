use axum::{
    extract::{Path, State},
    Json,
};
use repository::Repository;
mod request;
mod response;

use crate::{ApiResponse, IntoApiResponse};

use self::response::{Block, GetBlockResponse, GetBlocksResponse};

pub async fn get_blocks(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetBlocksResponse>> {
    let blocks = repo.block.find_all().await.into_response("502-003")?;

    let response = Json(GetBlocksResponse {
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
) -> ApiResponse<Json<GetBlockResponse>> {
    let block = repo
        .block
        .find_by_notion_page_id(id)
        .await
        .into_response("502-004")?;

    let Some(block) = block else {
        return Ok(Json(GetBlockResponse { block: None }));
    };

    Ok(Json(GetBlockResponse {
        block: Some(Block {
            parent_id: block.notion_page_id,
            contents: block.contents,
        }),
    }))
}
