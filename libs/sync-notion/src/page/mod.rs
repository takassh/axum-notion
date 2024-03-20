use crate::{Config, SyncNotionError};
use entities::page;
use notion_client::{
    endpoints::databases::query::request::{QueryDatabaseRequest, Sort, SortDirection, Timestamp},
    objects::page::Page,
};

use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::{error, info};

pub async fn spawn_service_to_get_pages(state: Arc<Config>) -> Result<(), SyncNotionError> {
    let (tx, rx) = mpsc::channel(100);

    sender(state.clone(), tx).await?;
    receiver(state.clone(), rx).await?;

    Ok(())
}

#[tracing::instrument]
async fn sender(state: Arc<Config>, tx: Sender<Vec<Page>>) -> Result<(), SyncNotionError> {
    tokio::spawn(async move {
        loop {
            let pages = scan_all_pages(state.clone()).await;

            info!("Complete scan pages");

            let result = tx.send(pages).await;
            if let Err(e) = result {
                error!("send: {}", e);
            }

            sleep(Duration::from_secs(state.pause_secs)).await;
        }
    });

    return Ok(());
}

#[tracing::instrument]
async fn scan_all_pages(state: Arc<Config>) -> Vec<Page> {
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
            .query_a_database(&state.notion_db_id, request.clone())
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
                error!("err: {:?}, request: {:?}", e, request);
            }
        }

        sleep(Duration::from_secs(state.pause_secs)).await;
    }

    pages
}

#[tracing::instrument]
async fn receiver(state: Arc<Config>, mut rx: Receiver<Vec<Page>>) -> Result<(), SyncNotionError> {
    tokio::spawn(async move {
        loop {
            let Some(pages) = rx.recv().await else {
                continue;
            };

            for page in pages {
                let json = serde_json::to_string_pretty(&page).unwrap();
                let model = page::Model {
                    notion_page_id: page.id.clone(),
                    contents: json,
                    created_at: page.created_time.naive_utc(),
                    ..Default::default()
                };

                let result = state.repository.page.save(model).await;
                if let Err(e) = result {
                    error!("save: {}", e);
                }
            }
        }
    });

    return Ok(());
}
