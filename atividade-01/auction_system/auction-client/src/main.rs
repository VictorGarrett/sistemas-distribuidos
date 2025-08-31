use lapin::{
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use futures_lite::stream::StreamExt;
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use rsa::Pkcs1v15Sign;

use crate::bid::Bid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");

    let channel = conn.create_channel().await?;

    let pem = fs::read_to_string("private_key.pem")?;
    let private_key = RsaPrivateKey::from_pkcs1_pem(pem)?;


    channel
        .exchange_declare(
            "leilao_iniciado",
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    
    let started_queue = channel
        .queue_declare(
            "", // random name
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let bid_queue = channel
        .queue_declare(
            "lance_realizado",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // bind the queue to the fanout exchange, to receive leilao_iniciado messages
    channel
        .queue_bind(
            started_queue.name().as_str(),
            "leilao_iniciado",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    
    let mut consumer = channel
        .basic_consume(
            started_queue.name().as_str(),
            "auction_client",
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


// make and publish a bid
async fn make_bid(auction_id: u32, client_id: u32, value: f64, bid_queue: &Queue, private_key: &RsaPrivateKey) -> Bid {
    
    let content = format!("{}:{}:{}", auction_id, client_id, value).into_bytes();
    
    let hashed = Sha256::digest(content);

    // sign
    let signature = private_key.sign(
        Pkcs1v15Sign::new_unprefixed(),
        &hashed,
    )?;
    
    let bid = Bid {
        aution_id,
        client_id,
        value,
        signature: general_purpose::STANDARD.encode(signature)
    };

    let payload = json!(bid).to_string();

    channel.basic_publish(
        "",
        "lance_realizado",
        BasicPublishOptions::default(),
        payload.as_bytes(),
        BasicProperties::default()
    ).await?.await?;
}