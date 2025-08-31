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

use crate::bid::Bid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");

    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "lance_realizado",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "lance_realizado",
            "bid-srv",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("Waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        println!("Received: {:?}", std::str::from_utf8(&delivery.data)?);
        delivery.ack(Default::default()).await?;
    }

    Ok(())
}

fn verify_bid(bid: &Bid, public_key: RsaPublicKey) -> bool {


    let content = format!("{}:{}:{}", bid.auction_id, bid.client_id, bid.value).into_bytes();
    
    let hashed = Sha256::digest(content);

    match public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &signature) {
        Ok(_) => return true,
        Err(e) => return false,
    }
}
