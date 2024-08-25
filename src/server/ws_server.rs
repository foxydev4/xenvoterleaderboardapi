use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::ws;

pub async fn run_ws_server(tx: broadcast::Sender<String>) {
    // Create the clients vector
    let clients = Arc::new(Mutex::new(Vec::new()));

    // Create the WebSocket filter with the Sender and Clients
    let ws_route = ws::ws_filter(clients.clone(), tx.clone());

    // Serve the WebSocket server on port 3030 (or another port of your choice)
    warp::serve(ws_route)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
