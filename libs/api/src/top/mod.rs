use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use repository::Repository;
use tokio::time::sleep;
use tracing::{error, info};

pub async fn send(
    ws: WebSocketUpgrade,
    State(repo): State<Repository>,
) -> Response {
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

        let text = msg.into_text();
        let Ok(text) = text else {
            error!(
                task = "msg.into_text",
                error = text.unwrap_err().to_string()
            );
            continue;
        };

        let t = top.set(&text);

        let Ok(t) = t else {
            error!(task = "top.set", error = t.unwrap_err().to_string());
            continue;
        };

        info!(task = "receive message", t = format!("{:?}", t));
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

    let (mut sender, _) = socket.split();

    loop {
        sleep(Duration::from_secs(10)).await;

        let result = top.get();

        let Ok(result) = result else {
            error!(task = "top.get", error = result.unwrap_err().to_string());
            return;
        };

        let result = serde_json::to_string(&result);
        let Ok(result) = result else {
            error!(task = "to_string", error = result.unwrap_err().to_string());
            return;
        };

        if let Err(e) = sender.send(Message::Text(result)).await {
            error!(task = "send", error = e.to_string());
        }
    }
}
