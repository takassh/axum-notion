use crate::State;
use anyhow::{anyhow, Context};
use cloudflare::models::text_embeddings::{
    TextEmbeddings, TextEmbeddingsRequest,
};
use futures::future::join_all;
use notion_client::{
    endpoints::databases::query::request::{
        QueryDatabaseRequest, Sort, SortDirection, Timestamp,
    },
    objects::{
        page::{Page, PageProperty},
        parent::Parent,
    },
};

use entity::{page::ParentType, post::Category, prelude::*};
use qdrant_client::{
    client::Payload,
    qdrant::{
        Condition, FieldCondition, Filter, Match, PointId, PointStruct, Value,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
    vec,
};
use tokio::{join, task::JoinHandle};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::error;

enum Message {
    Save { pages: Vec<Page> },
    Delete { page_ids: HashSet<String> },
}

pub fn spawn_service_to_get_pages(
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
            let state = state.clone();

            // For pages
            let notion_database_ids =
                state.repository.notion_database_id.find_all().await?;
            let all_page_ids: HashSet<String> = state
                .repository
                .page
                .find_all()
                .await?
                .into_iter()
                .map(|p| p.notion_page_id)
                .collect();
            let mut join_handles = vec![];
            for notion_database_id in notion_database_ids {
                let tx = tx.clone();
                let state = state.clone();
                let join_handle: JoinHandle<HashSet<String>> =
                    tokio::spawn(async move {
                        sleep(Duration::from_secs(state.pause_secs)).await;

                        let pages = scan_all_pages(
                            state.clone(),
                            &notion_database_id.id,
                        )
                        .await;

                        let result = tx
                            .send(Message::Save {
                                pages: pages.clone(),
                            })
                            .await;
                        if let Err(e) = result {
                            error!(
                                task = "scan all pages",
                                notion_database_id.id,
                                error = e.to_string(),
                            );
                        }

                        pages.into_iter().map(|p| p.id).collect()
                    });

                join_handles.push(join_handle);
            }

            sleep(Duration::from_secs(state.pause_secs)).await;

            // For static pages
            let notion_static_page_ids = state
                .repository
                .static_page
                .find_all()
                .await?
                .into_iter()
                .map(|p| p.notion_page_id);
            for notion_static_page_id in notion_static_page_ids.clone() {
                sleep(Duration::from_secs(state.pause_secs)).await;

                let page = state
                    .client
                    .pages
                    .retrieve_a_page(&notion_static_page_id, None)
                    .await;

                let Ok(page) = page else {
                    error!(
                        task = "retrieve_a_page",
                        notion_static_page_id,
                        error = page.unwrap_err().to_string(),
                    );
                    continue;
                };

                let result = tx.send(Message::Save { pages: vec![page] }).await;
                if let Err(e) = result {
                    error!(
                        task = "scan all static pages",
                        notion_static_page_id,
                        error = e.to_string(),
                    );
                }
            }

            // delete unused pages
            let results = join_all(join_handles).await;
            let mut new_page_ids = HashSet::new();
            for result in results.into_iter().flatten() {
                new_page_ids.extend(result);
            }
            for id in notion_static_page_ids {
                new_page_ids.insert(id);
            }

            let page_ids: HashSet<String> =
                all_page_ids.difference(&new_page_ids).cloned().collect();
            let result = tx
                .send(Message::Delete {
                    page_ids: page_ids.clone(),
                })
                .await;
            if let Err(e) = result {
                error!(
                    task = "send delete",
                    page_ids = format!("{:?}", page_ids),
                    error = e.to_string(),
                );
            }

            sleep(Duration::from_secs(state.pause_secs)).await;
        }
    })
}

async fn scan_all_pages(state: Arc<State>, page_id: &str) -> Vec<Page> {
    let mut next_cursor = None;
    let mut pages = vec![];
    loop {
        let request = QueryDatabaseRequest {
            sorts: Some(vec![Sort::Timestamp {
                timestamp: Timestamp::CreatedTime,
                direction: SortDirection::Ascending,
            }]),
            start_cursor: next_cursor.clone(),
            ..Default::default()
        };
        let response = state
            .client
            .databases
            .query_a_database(page_id, request.clone())
            .await;

        match response {
            Ok(mut response) => {
                pages.append(&mut response.results);

                if response.has_more {
                    next_cursor = response.next_cursor;
                } else {
                    break;
                }
            }
            Err(e) => {
                error!(
                    task = "query_a_database",
                    request = format!("{:?}", request),
                    error = e.to_string()
                );
            }
        }

        sleep(Duration::from_secs(state.pause_secs)).await;
    }

    pages
}

fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Message>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Some(Message::Save { pages }) => {
                    for page in pages {
                        let json = serde_json::to_string_pretty(&page).unwrap();
                        let parent_id = match page.parent {
                            Parent::DatabaseId { ref database_id } => {
                                database_id
                            }
                            Parent::PageId { ref page_id } => page_id,
                            _ => continue,
                        };

                        let stored =
                            state.repository.page.find_by_id(&page.id).await;

                        let Ok(stored) = stored else {
                            error!(
                                task = "find page by id",
                                model = format!("{:?}", page.id),
                                error = stored.unwrap_err().to_string()
                            );
                            continue;
                        };

                        let need_update = match stored {
                            None => true,
                            Some(stored) => {
                                let stored_contents =
                                    serde_json::from_str::<Page>(
                                        &stored.contents,
                                    );
                                let Ok(stored_page) = stored_contents else {
                                    error!(
                                        task = "desierialize stored page",
                                        parent_id,
                                        error = stored_contents
                                            .unwrap_err()
                                            .to_string(),
                                    );
                                    continue;
                                };

                                stored_page.last_edited_time
                                    != page.last_edited_time
                            }
                        };

                        if !need_update {
                            continue;
                        }

                        let _page = page.clone();
                        let page_model = PageEntity {
                            notion_page_id: _page.id.clone(),
                            notion_parent_id: parent_id.to_string(),
                            parent_type: match _page.parent {
                                Parent::DatabaseId { .. } => {
                                    ParentType::Database
                                }
                                Parent::PageId { .. } => ParentType::Page,
                                _ => ParentType::Database,
                            },
                            contents: json,
                            created_at: _page.created_time,
                            updated_at: None,
                        };

                        let post_model = PostEntity {
                            id: _page.id,
                            contents: None,
                            category: Category::Page,
                            created_at: _page.created_time,
                        };

                        let (
                            save_page_result,
                            save_post_result,
                            store_vector_result,
                        ) = join!(
                            state.repository.page.save(page_model.clone()),
                            state.repository.post.save(post_model.clone()),
                            store_vectors(
                                &state.cloudflare,
                                &state.qdrant,
                                state.collention.clone(),
                                page.clone()
                            ),
                        );

                        if let Err(e) = save_page_result {
                            error!(
                                task = "save page",
                                model = format!("{:?}", page_model),
                                error = e.to_string()
                            );
                        }

                        if let Err(e) = save_post_result {
                            error!(
                                task = "save post",
                                model = format!("{:?}", post_model),
                                error = e.to_string()
                            );
                        }

                        if let Err(e) = store_vector_result {
                            error!(
                                task = "store vector",
                                model = format!("{:?}", page),
                                error = e.to_string()
                            );
                        }
                    }
                }
                Some(Message::Delete { page_ids }) => {
                    for page_id in page_ids {
                        let (
                            delete_block_result,
                            delete_page_result,
                            vector_result,
                        ) = join!(
                            state.repository.block.delete_by_page_id(&page_id),
                            state.repository.page.delete(&page_id),
                            delete_vectors(
                                &state.qdrant,
                                state.collention.clone(),
                                &page_id,
                            )
                        );

                        if let Err(e) = delete_block_result {
                            error!(
                                task = "delete block",
                                page_id,
                                error = e.to_string()
                            );
                        }

                        if let Err(e) = delete_page_result {
                            error!(
                                task = "delete page",
                                page_id,
                                error = e.to_string()
                            );
                        }

                        if let Err(e) = vector_result {
                            error!(
                                task = "delete vector",
                                page_id,
                                error = e.to_string()
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    })
}

async fn delete_vectors(
    qdrant: &qdrant_client::client::QdrantClient,
    collection: String,
    page_id: &str,
) -> anyhow::Result<()> {
    qdrant.delete_points(
        collection.clone(),
        None,
        &qdrant_client::qdrant::PointsSelector {
            points_selector_one_of: Some(qdrant_client::qdrant::points_selector::PointsSelectorOneOf::Filter(Filter{
                should:vec![Condition{
                   condition_one_of:Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(FieldCondition{
                    key:"page_id".to_string(),
                    r#match:Some(Match{match_value:Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(page_id.to_string()))}),
                    ..Default::default()
                   }))
                }],
                ..Default::default()
            })),
        },
        None,
    ).await?;

    Ok(())
}

async fn store_vectors(
    cloudflare: &cloudflare::models::Models,
    qdrant: &qdrant_client::client::QdrantClient,
    collection: String,
    page: Page,
) -> anyhow::Result<()> {
    let page_id = page.id;
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
                }],
                ..Default::default()
            })),
        },
        None,
    ).await.context("failed to delete")?;

    let title = page
        .properties
        .get("title")
        .context("failed to get title")?;
    let PageProperty::Title { id: _, title } = title else {
        return Err(anyhow!("failed to get title"));
    };

    let title = title
        .iter()
        .flat_map(|t| t.plain_text())
        .collect::<Vec<_>>()
        .join("");

    let embedding = cloudflare
        .bge_small_en_v1_5(TextEmbeddingsRequest {
            text: title.as_str().into(),
        })
        .await
        .context(format!("failed to embed. {}", title))?;

    let Some(vectors) = embedding.result.data.first() else {
        return Err(anyhow!("failed to get vectors. {}", title));
    };

    let mut map = HashMap::new();
    map.insert("page_id".to_string(), Value::from(page_id.clone()));
    map.insert("document".to_string(), Value::from(title.clone()));

    let points = vec![PointStruct::new(
        PointId::from(page_id),
        vectors.clone(),
        Payload::new_from_hashmap(map),
    )];

    qdrant
        .upsert_points(collection.clone(), None, points, None)
        .await
        .context(format!("failed to upsert. {}", title))?;

    Ok(())
}
