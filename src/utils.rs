use ed25519_dalek::{Signature, VerifyingKey};
use hex::FromHex;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn verify(public_key: &str, message: &[u8], signatrue: &str) -> bool {
    let public_key_bytes = <[u8; 32]>::from_hex(public_key).expect("PUB Bytes Error");
    let signature_bytes = <[u8; 64]>::from_hex(signatrue).expect("SIGN Bytes Error");

    let signature = Signature::from_bytes(&signature_bytes);
    let verfiying_key = VerifyingKey::from_bytes(&public_key_bytes).expect("Invalid Verifying Key");

    match verfiying_key.verify_strict(&message, &signature) {
        Ok(_) => {
            tracing::debug!("🔎 OK: Verify");
            true
        }
        Err(e) => {
            tracing::debug!("🔎 ERROR: Verify - {}", e);
            false
        }
    }
}
