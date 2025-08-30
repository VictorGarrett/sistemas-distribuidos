use lapin::{
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use futures_lite::stream::StreamExt;
use rsa::{pkcs8::DecodePublicKey, traits::{PaddingScheme, SignatureScheme}, RsaPublicKey};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use rsa::Pkcs1v15Sign;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel.queue_declare("test_queue", QueueDeclareOptions::default(), FieldTable::default()).await?;

    let mut consumer = channel.basic_consume(
        "test_queue",
        "my_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default()
    ).await?;

    let pem = "-----BEGIN PUBLIC KEY-----
MIGeMA0GCSqGSIb3DQEBAQUAA4GMADCBiAKBgGmcA0BGDhveEY6+dmEo1lil2NpB
0Y8NpXdpBUi5DZLbk9Sg/sTv8z/AURr99a7MKAVFFngHTioOwxB5ruwhdvuFKKyq
TnzYK2dv87WJ7GqqUda2rlhBmy4CCOXSS+YLqgdQYj4QesBDOC9ojdFaIPGIyp77
J4iHAoICxN+y+Rn9AgMBAAE=
-----END PUBLIC KEY-----";

    let public_key = RsaPublicKey::from_public_key_pem(pem)?;

    println!("Waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let payload_str = std::str::from_utf8(&delivery.data)?;
        let json: Value = serde_json::from_str(payload_str)?;

        let message = json["message"].as_str().unwrap().as_bytes();
        let signature = general_purpose::STANDARD.decode(json["signature"].as_str().unwrap())?;

        // Hash the message
        let hashed = Sha256::digest(message);

        // Verify signature using PKCS1v15 padding with SHA-256
        match public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &signature) {
            Ok(_) => println!("✅ Verified message: {}", String::from_utf8_lossy(message)),
            Err(e) => println!("❌ Signature verification failed: {}", e),
        }

        delivery.ack(Default::default()).await?;
    }

    Ok(())
}