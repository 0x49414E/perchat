use std::sync::Arc;
use axum::{body::Body,
           extract::{State, WebSocketUpgrade,
                     ws::{WebSocket,Message}, Path},
           http::StatusCode,
           response::{IntoResponse, Response}};
use futures_util::{SinkExt, StreamExt};
use crate::{AppState,Client};

pub async fn websocket(ws: WebSocketUpgrade,
                       State(state): State<Arc<AppState>>) -> Response
{
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>)
{
    let (mut sender, mut receiver) = socket.split();

    let id_pub = String::new();

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(payload) = msg {
            let client : Client = match serde_json::from_str(&payload)
            {
                Ok(client) => client,
                Err(e) =>
                    {
                    let _ = sender.send(Message::Text("Failed to parse connect message".into())).await;
                    break;
                    }
            };

            //TODO!Push websocket and id_priv to hashmap
        }
    }

    let (mut tx, mut rx) = tokio::sync::broadcast::channel(100);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = {
        let tx = tx.clone();
        let id_pub = id_pub.clone();

        tokio::spawn(async move {
            while let Some(Ok(Message::Text(text))) = receiver.next().await {
                let _ = tx.send(format!("{}: {}", id_pub, text));
            }
        })
    };

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    //TODO!Format string and send it to the client

}