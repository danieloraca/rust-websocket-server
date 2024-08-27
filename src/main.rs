use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[derive(Clone)]
struct Client {
    name: String,
    sender: mpsc::UnboundedSender<Message>,
}

type Clients = Arc<RwLock<HashMap<Uuid, Client>>>;

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

    // Generate a unique name for the client
    let client_name = format!("Client-{}", Uuid::new_v4());

    // Create a client instance and add it to the list of connected clients
    let client = Client {
        name: client_name.clone(),
        sender: client_tx.clone(),
    };
    clients.write().await.insert(Uuid::new_v4(), client);

    // Task to receive messages from this client and broadcast them to others
    let clients_clone = clients.clone();
    tokio::spawn(async move {
        while let Some(result) = rx.next().await {
            if let Ok(msg) = result {
                let msg = Message::text(format!("{}: {}", client_name, msg.to_str().unwrap_or("")));
                broadcast_message(msg, &clients_clone).await;
            }
        }

        // Remove the client when they disconnect
        clients_clone
            .write()
            .await
            .retain(|_, c| c.name != client_name);
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

    for client in clients.values() {
        let _ = client.sender.send(msg.clone());
    }
}
