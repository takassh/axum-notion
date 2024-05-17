use axum::{
    extract::{Path, State},
    Json,
};
use repository::Repository;
mod request;
mod response;

use crate::response::{ApiResponse, IntoApiResponse};

use self::response::{BlockResp, GetBlockResp, GetBlocksResp};

pub async fn get_blocks(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetBlocksResp>> {
    let blocks = repo.block.find_all().await.into_response("502-003")?;

    let response = Json(GetBlocksResp {
        blocks: blocks
            .into_iter()
            .map(|a| BlockResp {
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
) -> ApiResponse<Json<GetBlockResp>> {
    let block = repo
        .block
        .find_by_notion_page_id(&id)
        .await
        .into_response("502-004")?;

    let Some(block) = block else {
        return Ok(Json(GetBlockResp { block: None }));
    };

    Ok(Json(GetBlockResp {
        block: Some(BlockResp {
            parent_id: block.notion_page_id,
            contents: block.contents,
        }),
    }))
}
