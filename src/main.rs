use futures_util::StreamExt;
use models::{Event, SocketMessage};
use serde_json;
use sled::open;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tracing_subscriber::FmtSubscriber;

mod events;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let db = open("experiment").unwrap();
    let address = "127.0.0.1:8080";
    let listener = TcpListener::bind(&address).await.expect("Cannot bind");

    tracing::info!("WebSocket server running on ws://{}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let db = db.clone();

        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during websocket handshake");

            tracing::info!("Client connected");

            let (mut write, mut read) = ws_stream.split();

            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    let message: SocketMessage =
                        serde_json::from_str(&msg.to_string()).expect("Invalid JSON");

                    match message.event.parse::<Event>() {
                        Ok(Event::GET) => events::get(&db, &mut write, &message).await,
                        Ok(Event::PUT) => events::put(&db, &mut write, &message).await,
                        Ok(Event::INVALID) | Err(_) => events::invalid(),
                    }
                }
            }
            tracing::info!("Client disconnected");
        });
    }
}
