use serde::Serialize;

#[derive(Serialize)]
pub struct Block {
    pub parent_id: String,
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetBlocksResponse {
    pub blocks: Vec<Block>,
}

#[derive(Serialize)]
pub struct GetBlockResponse {
    pub block: Option<Block>,
}
