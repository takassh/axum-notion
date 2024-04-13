use self::response::Event;
use crate::State;
use entity::{post::Category, prelude::*};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};
use tracing::error;

mod response;

pub fn spawn_service_to_get_events(
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
    tx: Sender<Vec<Event>>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let mut page = 1;
        loop {
            sleep(Duration::from_secs(state.config.pause_secs)).await;

            let result = state
                .client
                .get(
                    &format!("users/{}/events/public", state.config.username),
                    &[("per_page", 100), ("page", page)],
                )
                .await;

            let Ok((text, headers)) = result else {
                error!(
                    task = "load all events",
                    page,
                    err = result.unwrap_err().to_string(),
                );
                continue;
            };

            let events = serde_json::from_str::<Vec<Event>>(&text);
            let Ok(events) = events else {
                error!(
                    task = "load all events",
                    page,
                    err = events.unwrap_err().to_string(),
                );
                continue;
            };

            let result = tx.send(events).await;
            let Ok(_) = result else {
                error!(
                    task = "load all events",
                    page,
                    err = result.unwrap_err().to_string(),
                );
                continue;
            };

            let link = headers.get("link").unwrap().to_str().unwrap();
            if link.contains("rel=\"next\"") {
                page += 1;
            } else {
                page = 0;
            }
        }
    })
}

#[tracing::instrument]
fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Vec<Event>>,
) -> JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        loop {
            let Some(events) = rx.recv().await else {
                continue;
            };

            for event in events {
                let json = serde_json::to_string_pretty(&event).unwrap();
                let model = EventEntity {
                    github_event_id: event.id.clone(),
                    contents: json,
                    created_at: event.created_at,
                    ..Default::default()
                };

                let result = state.repository.event.save(model.clone()).await;
                if let Err(e) = result {
                    error!(
                        task = "save",
                        model = format!("{:?}", model),
                        error = e.to_string()
                    );
                }

                let model = PostEntity {
                    id: event.id,
                    contents: None,
                    category: Category::Event,
                    created_at: event.created_at,
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

#[cfg(test)]
mod test {
    use std::fs;

    use crate::{events::response::Event, util::workspace_dir};

    #[test]
    fn test_desilialize() {
        let dir = workspace_dir();

        // Arrange
        let text = fs::read_to_string(
            dir.join("libs/sync-github/src/events/test.json"),
        );

        // Act
        let events = serde_json::from_str::<Vec<Event>>(&text.unwrap());

        // Assert
        assert!(events.is_ok());
    }
}
