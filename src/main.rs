use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Clients = Arc<RwLock<Vec<mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    let clients = Clients::default();

    // WebSocket route
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    // Serve the WebSocket route
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

// Middleware to pass clients to each request
fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

// Handle the WebSocket connection
async fn handle_connection(ws: WebSocket, clients: Clients) {
    let (mut tx, mut rx) = ws.split();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();

    // Add this client to the list of connected clients
    clients.write().await.push(client_tx.clone());

    // Task to receive messages from this client and broadcast them to others
    let clients_clone = clients.clone();
    tokio::spawn(async move {
        while let Some(result) = rx.next().await {
            if let Ok(msg) = result {
                broadcast_message(msg, &clients_clone).await;
            }
        }
    });

    // Task to send messages from the broadcast channel to this client
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            if tx.send(msg).await.is_err() {
                break;
            }
        }
    });
}

// Broadcast a message to all connected clients
async fn broadcast_message(msg: Message, clients: &Clients) {
    let clients = clients.read().await;

    for client in clients.iter() {
        let _ = client.send(msg.clone());
    }
}
