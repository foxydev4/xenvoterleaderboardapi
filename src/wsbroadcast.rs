use futures_util::{stream::SplitSink, StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{self, Receiver, Sender};
use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::stream::SplitStream;

type Clients = Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>;

pub async fn handle_ws_connection(ws: WebSocket, clients: Clients, mut rx: Receiver<String>) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = async {
                while let Ok(msg) = rx.recv().await {
                    if ws_tx.send(Message::text(msg)).await.is_err() {
                        break;
                    }
                }
            } => {},
            _ = async {
                while ws_rx.next().await.is_some() {}
            } => {},
        }
    });

    clients.lock().unwrap().push(handle);
}

pub fn ws_filter(
    clients: Clients,
    tx: Sender<String>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::ws()
        .map(move |ws: warp::ws::Ws| {
            let clients = clients.clone();
            let rx = tx.subscribe();
            ws.on_upgrade(move |socket| handle_ws_connection(socket, clients, rx))
        })
}
