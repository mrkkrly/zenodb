use crate::models::{Data, Response, SocketMessage};
use crate::utils::{format_log, get_timestamp, verify};

use futures_util::{stream::SplitSink, SinkExt};
use sled::Db;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub async fn get(
    db: &Db,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message: &SocketMessage,
) {
    let mut response = Response {
        data: None,
        error: None,
    };
    match &message.identifier {
        Some(key) => match db.get(key) {
            Ok(Some(data)) => match String::from_utf8(data.to_vec()) {
                Ok(json_string) => {
                    let log = format_log("GET", key);
                    tracing::info!(log);

                    match serde_json::from_str::<Data>(&json_string) {
                        Ok(json) => {
                            response.data = Some(json);
                        }
                        Err(e) => {
                            response.error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    let log = format_log("GET", &e.to_string());
                    tracing::error!(log);
                    response.error = Some(e.to_string());
                }
            },
            Ok(None) => {
                let log = format_log("GET", &key.to_string());
                tracing::warn!(log);
                response.error = Some("404".to_string());
            }
            Err(e) => {
                let log = format_log("GET", &e.to_string());
                tracing::error!(log);
                response.error = Some(e.to_string());
            }
        },
        None => {
            let log = format_log("GET", "Missing identifier in message");
            tracing::warn!(log);
            response.error = Some("Missing identifier in message".to_string());
        }
    }

    let respsonse_message = Message::Text(serde_json::to_string(&response).expect("ERROR").into());

    if let Err(e) = write.send(respsonse_message).await {
        let log = format_log("GET", &e.to_string());
        tracing::error!(log);
    }
}

pub async fn put(
    db: &Db,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message: &SocketMessage,
) {
    let mut response = Response {
        data: None,
        error: None,
    };

    match (
        &message.identifier,
        &message.data,
        &message.public_key,
        &message.signature,
    ) {
        (Some(hash), Some(data), Some(public_key), Some(signature)) => {
            if verify(public_key, data.as_bytes(), &signature) {
                let signature_bs58 = bs58::encode(signature.as_bytes()).into_string();
                let public_key_bs58 = bs58::encode(public_key.as_bytes()).into_string();

                tracing::debug!("🔑 Base 58 Public Key - {}", public_key_bs58);

                // PREFIX from Base 58 Encoded Public Key
                let prefix = &public_key_bs58[0..32];

                let identifier = format!("{}-{}", prefix, hash);
                let timestamp = get_timestamp();

                let data = Data {
                    identifier: identifier.clone(),
                    data: data.to_string(),
                    signature: signature_bs58,
                    public_key: public_key_bs58,
                    timestamp: timestamp,
                };

                match serde_json::to_string(&response) {
                    Ok(json) => match db.insert(identifier.clone(), json.as_bytes()) {
                        Ok(_) => {
                            let log = format_log("PUT", &identifier);
                            tracing::info!(log);

                            response.data = Some(data);
                        }
                        Err(e) => {
                            let log = format_log("PUT", &e.to_string());
                            tracing::error!(log);

                            response.error = Some(e.to_string());
                        }
                    },
                    Err(e) => {
                        let log = format_log("PUT", &e.to_string());
                        tracing::error!(log);

                        response.error = Some(e.to_string());
                    }
                }
            } else {
                let log = format_log("PUT", "Failed to verify data");
                tracing::error!(log);

                response.error = Some("Failed to verify data".to_string());
            }
        }
        _ => {
            let log = format_log("PUT", "Missing identifier or data in message");
            tracing::warn!(log);
            response.error = Some("Missing identifier or data in message".to_string());
        }
    }

    let respsonse_message = Message::Text(serde_json::to_string(&response).expect("ERROR").into());

    if let Err(e) = write.send(respsonse_message).await {
        let log = format_log("PUT", &e.to_string());
        tracing::error!(log);
    }
}

pub fn invalid() {
    tracing::warn!("Invalid Event");
}
