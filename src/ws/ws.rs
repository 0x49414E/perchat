use std::ops::Deref;
use std::sync::Arc;
use axum::{body::Body,
           extract::{State, WebSocketUpgrade,
                     ws::{WebSocket,Message}, Path},
           http::StatusCode,
           response::{IntoResponse, Response}};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use crate::{AppState, ChatRoom, Client};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub async fn websocket(ws: WebSocketUpgrade, Path(id_priv) : Path<String>,
                       State(state): State<Arc<AppState>>) -> Response
{
    ws.on_upgrade(|socket| {
        handle_socket(socket, state)
    })
}

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>)
{
    let (mut sender, mut receiver) = socket.split();
    let mut id_pub = String::new();
    let mut tx = None::<tokio::sync::broadcast::Sender<String>>;


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

            id_pub = client.id_pub;

            let mut chats = (*state).chats.lock().unwrap();
            let chat = chats.entry(id_pub.clone()).or_insert_with(ChatRoom::new);

            tx = Some(chat.tx.clone());
        }
    }

    let mut tx = tx.unwrap();
    let mut rx = tx.subscribe();

    let msg = format!("{} joined.", id_pub.clone());
    let _ = tx.send(msg);

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
                let _ = tx.send(format!("{}: {}", id_pub.clone(), text));
            }
        })
    };

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

}