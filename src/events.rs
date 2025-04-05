use std::str::FromStr;

use crate::models::{FinalResponse, Response, SocketMessage};
use crate::utils::{get_timestamp, verify};

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
    let mut response = FinalResponse {
        data: None,
        error: None,
    };
    match &message.identifier {
        Some(key) => match db.get(key) {
            Ok(Some(data)) => match String::from_utf8(data.to_vec()) {
                Ok(json_string) => {
                    tracing::info!("GET - {}", key);

                    match serde_json::from_str::<Response>(&json_string) {
                        Ok(json) => {
                            response.data = Some(json);
                        }
                        Err(e) => {
                            response.error = Some(format!("{}", e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("GET - Failed to serialize response: {}", e);
                    response.error = Some(format!("{}", e.to_string()));
                }
            },
            Ok(None) => {
                tracing::warn!("GET - No data found for key: {}", key);
                response.error = Some(format!("No data found for key: {}", key));
            }
            Err(e) => {
                tracing::error!("GET - Database Error: {}", e);
                response.error = Some(format!("Database Error: {}", e));
            }
        },
        None => {
            tracing::warn!("GET - Missing identifier in message");
            response.error = Some(format!("Missing identifier in message"));
        }
    }

    let respsonse_message = Message::Text(serde_json::to_string(&response).expect("ERROR").into());

    if let Err(e) = write.send(respsonse_message).await {
        tracing::error!("GET - Failed to send response: {}", e);
    }
}

pub async fn put(
    db: &Db,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    message: &SocketMessage,
) {
    let mut response = FinalResponse {
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

                let data = Response {
                    identifier: identifier.clone(),
                    data: data.to_string(),
                    signature: signature_bs58,
                    public_key: public_key_bs58,
                    timestamp: timestamp,
                };

                match serde_json::to_string(&response) {
                    Ok(json) => match db.insert(identifier.clone(), json.as_bytes()) {
                        Ok(_) => {
                            tracing::info!("PUT - {}", identifier);
                            response.data = Some(data);
                        }
                        Err(e) => {
                            tracing::error!("PUT - Failed to store data: {}", e);
                            response.error = Some(format!("Failed to store data: {}", e));
                        }
                    },
                    Err(e) => {
                        tracing::error!("PUT - Failed to serialize data: {}", e);
                        response.error = Some(format!("Failed to serialize data: {}", e));
                    }
                }
            } else {
                tracing::error!("PUT - Failed to Verify Data!");
                response.error = Some(format!("Failed to Verify Data!"));
            }
        }
        _ => {
            tracing::warn!("PUT - Missing identifier or data in message");
            response.error = Some(format!("Missing identifier or data in message"));
        }
    }

    let respsonse_message = Message::Text(serde_json::to_string(&response).expect("ERROR").into());

    if let Err(e) = write.send(respsonse_message).await {
        tracing::error!("GET - Failed to send response: {}", e);
    }
}

pub fn invalid() {
    tracing::warn!("Invalid Event");
}
