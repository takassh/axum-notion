use serde::Serialize;

#[derive(Serialize)]
pub struct BlockResp {
    pub parent_id: String,
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetBlocksResp {
    pub blocks: Vec<BlockResp>,
}

#[derive(Serialize)]
pub struct GetBlockResp {
    pub block: Option<BlockResp>,
}
