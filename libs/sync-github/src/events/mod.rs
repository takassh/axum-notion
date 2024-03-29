use self::response::Event;
use crate::State;
use entity::prelude::*;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::sleep,
};
use tracing::{error, info};

mod response;

pub async fn spawn_service_to_get_events(
    state: Arc<State>,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(100);

    let _ = sender(state.clone(), tx);
    let _ = receiver(state.clone(), rx);

    Ok(())
}

#[tracing::instrument]
fn sender(state: Arc<State>, tx: Sender<Vec<Event>>) -> anyhow::Result<()> {
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
                error!("get: {}", result.unwrap_err());
                continue;
            };

            let events = serde_json::from_str::<Vec<Event>>(&text);
            let Ok(events) = events else {
                error!("parse: {}", events.err().unwrap());
                continue;
            };

            let result = tx.send(events).await;
            let Ok(_) = result else {
                error!("send: {}", result.err().unwrap());
                continue;
            };

            let link = headers.get("link").unwrap().to_str().unwrap();
            if link.contains("rel=\"next\"") {
                page += 1;
            } else {
                info!("Finish to load all events");
                page = 0;
            }
        }
    });

    return Ok(());
}

#[tracing::instrument]
fn receiver(
    state: Arc<State>,
    mut rx: Receiver<Vec<Event>>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        loop {
            let Some(events) = rx.recv().await else {
                continue;
            };

            for event in events {
                let json = serde_json::to_string_pretty(&event).unwrap();
                let model = EventEntity {
                    github_event_id: event.id,
                    contents: json,
                    created_at: event.created_at,
                    ..Default::default()
                };

                let result = state.repository.event.save(model).await;
                if let Err(e) = result {
                    error!("receiver: {}", e);
                }
            }
        }
    });

    return Ok(());
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
