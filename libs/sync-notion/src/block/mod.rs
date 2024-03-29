use crate::State;
use async_recursion::async_recursion;
use entity::prelude::*;
use notion_client::objects::block::{Block, BlockType};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::{error, info};

struct Message {
    parent_id: String,
    blocks: Vec<Block>,
}

pub async fn spawn_service_to_get_blocks(
    state: Arc<State>,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(100);

    let _ = sender(state.clone(), tx);
    let _ = receiver(state.clone(), rx);

    Ok(())
}

#[tracing::instrument]
fn sender(state: Arc<State>, tx: Sender<Message>) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            let pages = state.repository.page.find_all().await;

            if let Err(e) = &pages {
                error!("failed to find all: {}", e);
            }
            let pages = pages.unwrap();

            for page in pages {
                info!("page id {} starts", page.notion_page_id);

                let _children =
                    get_children(state.clone(), &page.notion_page_id).await;

                let mut children = vec![];
                for _child in _children {
                    let child = scan_block(state.clone(), _child).await;
                    children.push(child);
                }

                let result = tx
                    .send(Message {
                        parent_id: page.notion_page_id.to_owned(),
                        blocks: children,
                    })
                    .await;
                if let Err(e) = result {
                    error!("send: {}", e);
                }
            }
        }
    });

    Ok(())
}

#[async_recursion]
async fn scan_block(state: Arc<State>, mut block: Block) -> Block {
    let Some(id) = &block.id else {
        return block;
    };
    let Some(has_children) = block.has_children else {
        return block;
    };
    if !has_children {
        return block;
    }

    let _children = get_children(state.clone(), id).await;

    let mut children = vec![];
    for _child in _children {
        let child = scan_block(state.clone(), _child).await;
        children.push(child);
    }

    block.block_type = match block.block_type {
        BlockType::BulletedListItem {
            mut bulleted_list_item,
        } => {
            bulleted_list_item.children = Some(children);
            BlockType::BulletedListItem { bulleted_list_item }
        }
        BlockType::NumberedListItem {
            mut numbered_list_item,
        } => {
            numbered_list_item.children = Some(children);
            BlockType::NumberedListItem { numbered_list_item }
        }
        BlockType::Table { mut table } => {
            table.children = Some(children);
            BlockType::Table { table }
        }
        BlockType::Template { mut template } => {
            template.children = Some(children);
            BlockType::Template { template }
        }
        BlockType::ToDo { mut to_do } => {
            to_do.children = Some(children);
            BlockType::ToDo { to_do }
        }
        BlockType::Toggle { mut toggle } => {
            toggle.children = Some(children);
            BlockType::Toggle { toggle }
        }
        t => t,
    };

    return block;
}

#[tracing::instrument]
async fn get_children(state: Arc<State>, parent_block_id: &str) -> Vec<Block> {
    let mut next_cursor: Option<String> = None;
    let mut blocks = vec![];
    loop {
        sleep(Duration::from_secs(state.pause_secs)).await;

        let response = state
            .client
            .blocks
            .retrieve_block_children(
                parent_block_id,
                next_cursor.as_deref(),
                None,
            )
            .await;

        match response {
            Ok(mut response) => {
                blocks.append(&mut response.results);

                if response.has_more {
                    next_cursor = response.next_cursor;
                } else {
                    break;
                }
            }
            Err(e) => {
                error!("err: {:?}", e);
            }
        }
    }

    blocks
}

#[tracing::instrument]
fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Message>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            let Some(message) = rx.recv().await else {
                continue;
            };

            let parent_id = &message.parent_id;
            let json = serde_json::to_string_pretty(&message.blocks).unwrap();
            let model = BlockEntity {
                notion_page_id: parent_id.to_string(),
                contents: json,
                ..Default::default()
            };

            let result = state.repository.block.save(model).await;
            if let Err(e) = result {
                error!("save: {}", e);
            }
        }
    });
    return Ok(());
}
