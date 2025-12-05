use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::database::Database;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { channel: String },
    
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channel: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "connected")]
    Connected { message: String },
    
    #[serde(rename = "hot-accounts-update")]
    HotAccountsUpdate { data: Vec<HotAccountData> },
    
    #[allow(dead_code)]
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Serialize)]
struct HotAccountData {
    pubkey: String,
    contention_score: f64,
    lock_attempts: i64,
    avg_priority_fee: i64,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(db): State<Database>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, db))
}

async fn handle_socket(socket: WebSocket, db: Database) {
    let (mut sender, mut receiver) = socket.split();
    
    // Send welcome message
    let welcome = ServerMessage::Connected {
        message: "Connected to Solana Lock Dashboard".to_string(),
    };
    
    if let Ok(msg) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(msg)).await;
    }

    // Spawn a task to send periodic updates
    let db_clone = db.clone();
    let mut send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Fetch hot accounts
            match db_clone.get_hot_accounts(20, 5).await {
                Ok(accounts) => {
                    let data: Vec<HotAccountData> = accounts
                        .iter()
                        .map(|acc| HotAccountData {
                            pubkey: acc.account_pubkey.clone(),
                            contention_score: acc.avg_contention.unwrap_or(0.0),
                            lock_attempts: acc.lock_attempts,
                            avg_priority_fee: acc.avg_priority_fee.unwrap_or(0.0) as i64,
                        })
                        .collect();
                    
                    let update = ServerMessage::HotAccountsUpdate { data };
                    
                    if let Ok(msg) = serde_json::to_string(&update) {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error fetching hot accounts: {}", e);
                }
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Subscribe { channel } => {
                            info!("Client subscribed to channel: {}", channel);
                        }
                        ClientMessage::Unsubscribe { channel } => {
                            info!("Client unsubscribed from channel: {}", channel);
                        }
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    info!("WebSocket connection closed");
}
