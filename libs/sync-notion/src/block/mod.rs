use crate::State;
use anyhow::Context;
use async_recursion::async_recursion;
use cloudflare::models::text_embeddings::{
    TextEmbeddings, TextEmbeddingsRequest,
};
use entity::prelude::*;
use notion_client::objects::block::{Block, BlockType};
use qdrant_client::{
    client::Payload,
    qdrant::{Condition, FieldCondition, Filter, Match, PointStruct, Value},
};

use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    join,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};
use tracing::error;
use uuid::Uuid;

struct Message {
    parent_id: String,
    blocks: Vec<Block>,
}

pub fn spawn_service_to_get_blocks(
    state: Arc<State>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let (tx, rx) = mpsc::channel(100);

    let sender_handler = sender(state.clone(), tx);
    let receiver_handler = receiver(state.clone(), rx);

    vec![sender_handler, receiver_handler]
}

fn sender(
    state: Arc<State>,
    tx: Sender<Message>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            let pages = state.repository.page.find_all().await;

            let Ok(pages) = pages else {
                error!(task = "find all", err = pages.unwrap_err().to_string());
                continue;
            };

            for page in pages {
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
                    error!(
                        task = "load all blocks",
                        page.notion_page_id,
                        error = e.to_string(),
                    );
                }
            }
        }
    })
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
                error!(
                    task = "retrieve block children",
                    parent_block_id,
                    error = e.to_string()
                );
                break;
            }
        }
    }

    blocks
}

fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Message>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            let Some(message) = rx.recv().await else {
                continue;
            };

            let parent_id = &message.parent_id;

            // Check if this page has been updated
            let mut last_edited_time_map = HashMap::new();
            for block in &message.blocks {
                if let Some(edited_at) = block.last_edited_time {
                    last_edited_time_map.insert(block.id.clone(), edited_at);
                }
            }

            let Ok(stored) = state
                .repository
                .block
                .find_by_notion_page_id(parent_id)
                .await
            else {
                continue;
            };

            let need_update = match stored {
                None => true,
                Some(stored) => {
                    let stored_contents =
                        serde_json::from_str::<Vec<Block>>(&stored.contents);
                    let Ok(stored_blocks) = stored_contents else {
                        error!(
                            task = "desierialize stored blocks",
                            parent_id,
                            error = stored_contents.unwrap_err().to_string(),
                        );
                        continue;
                    };

                    let mut need_update = false;
                    for stored_block in stored_blocks {
                        if let Some(last_edited_at) =
                            last_edited_time_map.get(&stored_block.id)
                        {
                            if *last_edited_at
                                > stored_block
                                    .last_edited_time
                                    .unwrap_or_default()
                            {
                                need_update = true;
                                break;
                            }
                        } else {
                            need_update = true;
                            break;
                        }
                        last_edited_time_map.remove(&stored_block.id);
                    }

                    need_update || !last_edited_time_map.is_empty()
                }
            };

            if !need_update {
                continue;
            }

            let json = serde_json::to_string_pretty(&message.blocks).unwrap();
            let model = BlockEntity {
                notion_page_id: parent_id.to_string(),
                contents: json,
                ..Default::default()
            };

            let (save_result, store_result) = join!(
                state.repository.block.save(model),
                store_vectors(
                    &state.cloudflare,
                    &state.qdrant,
                    state.collention.clone(),
                    message.blocks,
                    parent_id,
                )
            );

            if let Err(e) = save_result {
                error!(
                    task = "save all blocks",
                    parent_id,
                    error = e.to_string(),
                );
            }

            if let Err(e) = store_result {
                error!(
                    task = "store vectors",
                    parent_id,
                    error = e.to_string(),
                );
            }
        }
    })
}

async fn store_vectors(
    cloudflare: &cloudflare::models::Models,
    qdrant: &qdrant_client::client::QdrantClient,
    collection: String,
    blocks: Vec<Block>,
    page_id: &str,
) -> anyhow::Result<()> {
    qdrant.delete_points(
        collection.clone(),
        None,
        &qdrant_client::qdrant::PointsSelector {
            points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(Filter{
                must:vec![Condition{
                   condition_one_of:Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(FieldCondition{
                    key:"page_id".to_string(),
                    r#match:Some(Match{match_value:Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(page_id.to_string()))}),
                    ..Default::default()
                   }))
                },Condition{
                    condition_one_of:Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(FieldCondition{
                     key:"type".to_string(),
                     r#match:Some(Match{match_value:Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(serde_json::to_string(&DocumentTypeEntity::Block).unwrap()))}),
                     ..Default::default()
                    }))
                 }],
                ..Default::default()
            })),
        },
        None,
    ).await.context("failed to delete")?;

    let mut blocks = blocks
        .into_iter()
        .flat_map(|block| block.block_type.plain_text())
        .flatten();

    loop {
        let mut texts = String::new();
        loop {
            let Some(text) = blocks.next() else {
                return Ok(());
            };
            if text.is_empty() {
                continue;
            }

            texts.push_str(&format!(" {}", text.as_str()));

            if texts.len() > 600 {
                break;
            }
        }

        let embedding = cloudflare
            .bge_small_en_v1_5(TextEmbeddingsRequest {
                text: texts.as_str().into(),
            })
            .await
            .context(format!("failed to embed. {}", texts))?;

        let Some(vectors) = embedding.result.data.first() else {
            continue;
        };

        let mut map = HashMap::new();
        map.insert("page_id".to_string(), Value::from(page_id));
        map.insert("document".to_string(), Value::from(texts.clone()));
        map.insert(
            "type".to_string(),
            Value::from(
                serde_json::to_string(&DocumentTypeEntity::Block).unwrap(),
            ),
        );

        let points = vec![PointStruct::new(
            Uuid::new_v4().hyphenated().to_string(),
            vectors.clone(),
            Payload::new_from_hashmap(map),
        )];

        qdrant
            .upsert_points(collection.clone(), None, points, None)
            .await
            .context(format!("failed to upsert. {}", texts))?;
    }
}
