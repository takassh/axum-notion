use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use repositories::Repository;
mod request;
mod response;

use crate::util::into_response;

use self::response::{Block, GetBlockResponse, GetBlocksResponse};

pub async fn get_blocks(
    State(repo): State<Repository>,
) -> Result<Json<GetBlocksResponse>, Response> {
    let blocks = repo
        .block
        .find_all()
        .await
        .map_err(|e| into_response(e, "find all"))?;

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
) -> Result<Json<GetBlockResponse>, Response> {
    let block = repo
        .block
        .find_by_notion_page_id(id)
        .await
        .map_err(|e| into_response(e, "find by notion page id"))?;

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
