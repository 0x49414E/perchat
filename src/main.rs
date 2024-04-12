use std::collections::{HashMap, HashSet, BTreeMap};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::env;
use std::future::Future;
use std::time::Duration;
use std::ops::Deref;
use axum::{
    extract::{Path, Json, Query, rejection::JsonRejection, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    http::{Method, StatusCode},
    response::{IntoResponse, Response, Html},
    routing::{get, post},
    Router,
    body::{Body, Bytes}
};
use axum_server::Handle;
use tokio::{sync::broadcast, signal};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, Error};
use tower_http::cors::{CorsLayer,Any};
use log::warn;
use dotenv::dotenv;

pub mod ws;
pub mod axum_handlers;

use axum_handlers::Decryptor;

use ws::handler;
use axum_handlers::generate_id;

static SERVER_SECRET: &str = "2297dqzV55P5KK9S";

#[derive(Deserialize, Serialize)]
struct Client {
    id_priv: String,
    id_pub: String,
}

struct AppState {
    chats: Mutex<HashMap<String,WebSocket>>,
    server_key: String,
    decryptor: Mutex<Arc<Decryptor>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let server_key = std::env::var("SERVER_KEY").unwrap_or_else(|err| SERVER_SECRET.to_string());

    let addr = SocketAddr::new(IpAddr::from([0,0,0,0]), 2020);
    let app_state = Arc::new(AppState {
        chats: Mutex::new(HashMap::new()),
        server_key,
        decryptor: Mutex::new(Arc::new(Decryptor::new())),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/generate_id", get(generate_id))
        .route("/events/:id", get(handler::websocket))
        .with_state(app_state)
        .layer(cors);

    let handle = Handle::new();
    tokio::spawn(shutdown(handle.clone()));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn shutdown(h: Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    let sig: &str;

    tokio::select! {
        _ = ctrl_c => { sig = "Ctrl+C"; },
        _ = terminate => { sig = "SIGTERM"; },
    }

    warn!("Received {}, shutting down...", sig);
    h.graceful_shutdown(Some(Duration::from_secs(30)));
}
