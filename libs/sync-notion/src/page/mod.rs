use crate::State;
use futures::future::join_all;
use notion_client::{
    endpoints::databases::query::request::{
        QueryDatabaseRequest, Sort, SortDirection, Timestamp,
    },
    objects::{page::Page, parent::Parent},
};

use entity::{post::Category, prelude::*};
use std::{collections::HashSet, sync::Arc, time::Duration, vec};
use tokio::task::JoinHandle;
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

            let results = join_all(join_handles).await;
            let mut new_page_ids = HashSet::new();
            for result in results.into_iter().flatten() {
                new_page_ids.extend(result);
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

                        let result =
                            state.repository.page.save(model.clone()).await;
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

                        let result =
                            state.repository.post.save(model.clone()).await;
                        if let Err(e) = result {
                            error!(
                                task = "save",
                                model = format!("{:?}", model),
                                error = e.to_string()
                            );
                        }
                    }
                }
                Some(Message::Delete { page_ids }) => {
                    for page_id in page_ids {
                        let result =
                            state.repository.page.delete(&page_id).await;
                        if let Err(e) = result {
                            error!(
                                task = "delete",
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
