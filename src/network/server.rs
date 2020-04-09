use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{mpsc, Mutex, oneshot};
use warp::ws::WebSocket;
use warp::Filter;
use warp::ws::Message;
use crate::network::packet::NetworkHandler;
use crate::env::environment::Environment;

pub async fn host(env: Environment, port: u16) {
    static INDEX: &str = include_str!("index.html");

    let network_handler = Arc::new(Mutex::new(NetworkHandler::new(Box::new(env))));

    let game_hook = warp::path("rews")
            .and(warp::ws())
            .and(warp::any().map(move || network_handler.clone()))
            .map(|ws: warp::ws::Ws, network_handler: Arc<Mutex<NetworkHandler>>| {
                ws.on_upgrade(move |socket| handle_connection(socket, network_handler))
            });
    
    let index = warp::path::end().map(|| warp::reply::html(INDEX));

    let routes = index.or(game_hook);

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (_addr, server) = warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
        shutdown_rx.await.ok();
    });

    tokio::task::spawn(server);
}

async fn handle_connection(socket: WebSocket, network_handler: Arc<Mutex<NetworkHandler>>) {
    let (ws_tx, mut ws_rx) = socket.split();

    // Setup a message sending channel for the handler
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.map(|message| Ok(message)).forward(ws_tx));

    let id: usize;
    let handler_tx: mpsc::UnboundedSender<(usize, Message)>;

    let mut handler_mut = network_handler.lock().await;
    match handler_mut.add_client(tx) {
        Some(uid) => {
            id = uid;
            handler_tx = handler_mut.get_sender();
            drop(handler_mut);
        }
        None => {
            drop(handler_mut);
            return;
        }
    }

    // Pipe incoming messages to the network handler
    while let Some(result) = ws_rx.next().await {
        let message = match result {
            Ok(message) => message,
            Err(e) => {
                eprintln!("WS error {}", e);
                break;
            }
        };

        if let Err(e) = handler_tx.send((id, message)) {
            eprintln!("Failed to pipe WS message to handler, {}", e);
        }
    }

    network_handler.lock().await.remove_client(id);
}