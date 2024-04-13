use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{SinkExt, StreamExt};
use repository::Repository;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

pub async fn send(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    State(repo): State<Repository>,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` connected.");
    ws.on_upgrade(|socket| handle_send_socket(socket, repo))
}

async fn handle_send_socket(socket: WebSocket, repo: Repository) {
    let Some(top) = repo.top else {
        println!("Cache is not enabled");
        return;
    };

    let (_, mut receiver) = socket.split();

    loop {
        let Some(msg) = receiver.next().await else {
            continue;
        };

        let Ok(msg) = msg else {
            error!(
                task = "receive message",
                error = msg.unwrap_err().to_string()
            );
            return;
        };

        match msg {
            Message::Close(_) => {
                info!("Connection closed");
                return;
            }
            Message::Text(text) => {
                let t = top.set(&text);

                let Ok(t) = t else {
                    error!(
                        task = "top.set",
                        error = t.unwrap_err().to_string()
                    );
                    continue;
                };

                info!(task = "receive message", t = format!("{:?}", t));
            }
            _ => {}
        }
    }
}

pub async fn receive(
    ws: WebSocketUpgrade,
    State(repo): State<Repository>,
) -> Response {
    ws.on_upgrade(|socket| handle_receive_socket(socket, repo))
}

async fn handle_receive_socket(socket: WebSocket, repo: Repository) {
    let Some(top) = repo.top else {
        println!("Cache is not enabled");
        return;
    };

    let (mut sender, mut receiver) = socket.split();
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    tokio::spawn(async move {
        loop {
            select! {
                _ = cloned_token.cancelled() => {
                    info!("token is cancelled");
                    return;
                }
                _ = sleep(Duration::from_secs(10)) => {

                    let result = top.get();

                    let Ok(result) = result else {
                        error!(
                            task = "top.get",
                            error = result.unwrap_err().to_string()
                        );
                        return;
                    };

                    let result = serde_json::to_string(&result);
                    let Ok(result) = result else {
                        error!(
                            task = "to_string",
                            error = result.unwrap_err().to_string()
                        );
                        return;
                    };

                    if let Err(e) = sender.send(Message::Text(result)).await {
                        error!(task = "send", error = e.to_string());
                    }
                }
            }
        }
    });

    loop {
        let Some(msg) = receiver.next().await else {
            continue;
        };

        let Ok(msg) = msg else {
            error!(
                task = "receive message",
                error = msg.unwrap_err().to_string()
            );
            return;
        };

        if let Message::Close(_) = msg {
            info!("Connection closed");
            token.cancel();
            return;
        }
    }
}
