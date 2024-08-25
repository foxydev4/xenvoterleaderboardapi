use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{Sender};
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Clients = Arc<Mutex<Vec<tokio::sync::broadcast::Sender<String>>>>;

pub async fn handle_ws_connection(ws: WebSocket, clients: Clients, tx: Sender<String>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe(); // Create a new receiver for this connection

    {
        let mut clients = clients.lock().unwrap();
        clients.push(tx.clone());
    }

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while ws_rx.next().await.is_some() {}

    // Remove the client upon disconnection
    let mut clients = clients.lock().unwrap();
    clients.retain(|client| {
        // In the current setup, you would need a way to determine if the client is still active.
        // This example assumes all clients are active, as `Sender` does not have a `is_closed` method.
        // For actual connection tracking, you would need to maintain a list of WebSocket connections
        // and handle them accordingly.
        true // Replace with actual client status check if possible
    });
}

pub fn ws_filter(
    clients: Clients,
    tx: Sender<String>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::ws()
        .map(move |ws: warp::ws::Ws| {
            let clients = clients.clone();
            let tx = tx.clone(); // Clone the sender to create a new receiver for each connection
            ws.on_upgrade(move |socket| handle_ws_connection(socket, clients, tx))
        })
}
