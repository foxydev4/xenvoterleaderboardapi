
use tokio::sync::broadcast;
use mongodb::{Client};

mod ws;
mod models;
mod server;
mod fetch;
mod routes;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Create a MongoDB client
    let mongo_uri = "mongodb://localhost:27017";
    let client = Client::with_uri_str(mongo_uri).await.unwrap();
    let db = client.database("block_data");
    let collection = db.collection("blocks");

    // Create a broadcast channel
    let (tx, _rx) = broadcast::channel(100);

    // Start the WebSocket server in a separate task
    tokio::spawn(server::ws_server::run_ws_server(tx.clone()));

    // Start the HTTP REST server in a separate task
    tokio::spawn(server::http_server::run_http_server(collection.clone()));

    // Start fetching, broadcasting data, and saving to the database
    fetch::fetch_data_and_broadcast(tx, collection).await;
}