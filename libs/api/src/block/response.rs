use serde::Serialize;

#[derive(Serialize)]
pub struct Block {
    pub parent_id: String,
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetBlocksRespose {
    pub blocks: Vec<Block>,
}

#[derive(Serialize)]
pub struct GetBlockRespose {
    pub block: Option<Block>,
}
