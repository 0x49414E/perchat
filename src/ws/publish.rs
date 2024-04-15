use std::ops::Deref;
use std::sync::Arc;
use axum::{body::Body,
           extract::{State, WebSocketUpgrade, Json,
                     ws::{WebSocket,Message}, Path},
           http::StatusCode,
           response::{IntoResponse, Response}};
use serde_json::{json, Value};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize,Serialize};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use crate::{AppState, ChatRoom, Client};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Deserialize, Serialize)]
struct Payload {
    id_priv: String,
    id_pub: String,
    dest_id_pub: String,
    msg: String,
}

pub async fn send( State(state) : State<Arc<AppState>>, Json(json) : Json<Value>) -> (StatusCode, Json<Value>)
{
    let payload : Payload = match serde_json::from_value(json) {
        Ok(payload) => payload,
        Err(e) => { return (StatusCode::NOT_ACCEPTABLE, Json(json!({"ERROR": "Parameters missing"})) )
        },
    };

    let dest_id_pub = payload.dest_id_pub.clone();

    let state = (*state).chats.lock().unwrap();
    let chat = state.get(&dest_id_pub).unwrap();

    let tx = chat.tx.clone();

    let _ = tx.send(format!("{}: {}", payload.id_pub.clone(), payload.msg.clone()));

    (StatusCode::OK, Json(serde_json::to_value(payload).unwrap()))
}