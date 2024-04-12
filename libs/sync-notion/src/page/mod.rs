use crate::State;
use futures::future::join_all;
use notion_client::{
    endpoints::databases::query::request::{
        QueryDatabaseRequest, Sort, SortDirection, Timestamp,
    },
    objects::{page::Page, parent::Parent},
};

use entity::{post::Category, prelude::*};
use std::{sync::Arc, time::Duration, vec};
use tokio::task::JoinHandle;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::{error, info};

pub fn spawn_service_to_get_pages(
    state: Arc<State>,
) -> Vec<JoinHandle<anyhow::Result<()>>> {
    let (tx, rx) = mpsc::channel(100);

    let sender_handler = sender(state.clone(), tx);
    let receiver_handler = receiver(state.clone(), rx);

    vec![sender_handler, receiver_handler]
}

#[tracing::instrument]
fn sender(
    state: Arc<State>,
    tx: Sender<Vec<Page>>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            let state = state.clone();
            let notion_database_ids =
                state.repository.notion_database_id.find_all().await?;

            let mut join_handles = vec![];

            for notion_database_id in notion_database_ids {
                let tx = tx.clone();
                let state = state.clone();
                let join_handle = tokio::spawn(async move {
                    sleep(Duration::from_secs(state.pause_secs)).await;

                    let pages =
                        scan_all_pages(state.clone(), &notion_database_id.id)
                            .await;

                    let result = tx.send(pages).await;
                    if let Err(e) = result {
                        error!(
                            task = "scan all pages",
                            notion_database_id.id,
                            error = e.to_string(),
                        );
                    } else {
                        info!(task = "scan all pages", notion_database_id.id);
                    }
                });

                join_handles.push(join_handle);
            }

            join_all(join_handles).await;

            sleep(Duration::from_secs(state.pause_secs)).await;
        }
    })
}

#[tracing::instrument]
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

#[tracing::instrument]
fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Vec<Page>>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            let Some(pages) = rx.recv().await else {
                continue;
            };

            for page in pages {
                let json = serde_json::to_string_pretty(&page).unwrap();
                let parent_id = match page.parent {
                    Parent::DatabaseId { database_id } => database_id,
                    _ => continue,
                };
                let model = PageEntity {
                    notion_page_id: page.id.clone(),
                    notion_database_id: parent_id,
                    contents: json,
                    created_at: page.created_time,
                    ..Default::default()
                };

                let result = state.repository.page.save(model.clone()).await;
                if let Err(e) = result {
                    error!(
                        task = "save",
                        model = format!("{:?}", model),
                        error = e.to_string()
                    );
                }

                let model = PostEntity {
                    id: page.id,
                    contents: None,
                    category: Category::Page,
                    created_at: page.created_time,
                };

                let result = state.repository.post.save(model.clone()).await;
                if let Err(e) = result {
                    error!(
                        task = "save",
                        model = format!("{:?}", model),
                        error = e.to_string()
                    );
                }
            }
        }
    })
}
