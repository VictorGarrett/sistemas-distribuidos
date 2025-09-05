use lapin::options::{QueueDeclareOptions, QueueBindOptions, ExchangeDeclareOptions};
use lapin::{
    Channel, Connection, ConnectionProperties
};
use lapin::types::FieldTable;

use std::sync::{Arc, mpsc};
use std::error::Error;

use tokio::{task::JoinHandle};

use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
use rsa::Pkcs1v15Sign;

pub mod models;
pub mod tasks;
pub mod client;
pub mod cli;
pub mod command;

use crate::command::Command;

use crate::tasks::{
    task_cli, task_leilao_iniciado, task_publish_cmd, task_receive_notification
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");
    let conn = Arc::new(conn);

    let (notification_queue_name, leilao_iniciado_mq) = init_rabbitmq_structs(conn.clone()).await?;

    // let pem = fs::read_to_string("private_key.pem")?;
    // let private_key = RsaPrivateKey::from_pkcs1_pem(pem)?;

    let handles = init_tasks(
        conn, 
        notification_queue_name,
        leilao_iniciado_mq
    );

    tokio::join!(handles);

    Ok(())
}

async fn init_tasks(
    conn: Arc<Connection>,
    notification_queue_name: String,
    leilao_iniciado_mq: String
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();

    let (sender, receiver) = mpsc::channel::<Command>();

    handles.push(tokio::task::spawn_blocking(|| {
        task_cli(sender)
    }));

    handles.push(tokio::spawn(task_receive_notification(
        conn.clone(),
        notification_queue_name
    )));

    handles.push(tokio::spawn(task_publish_cmd(
        conn.clone(),
        receiver
    )));

    handles.push(tokio::spawn(task_leilao_iniciado(
        conn.clone(), 
        leilao_iniciado_mq,
        "client_0".to_string()
    )));

    handles
}

async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<(String, String), Box<dyn std::error::Error>> {
    let channel = conn.create_channel().await?;

    let started_queue_name = init_leilao_iniciado(&channel).await?;
    let notification_queue_name = init_receive_notification(&channel).await?;
    init_process_input(&channel).await?;

    Ok((notification_queue_name, started_queue_name))
}

async fn init_leilao_iniciado(channel: &Channel) -> Result<String, Box<dyn std::error::Error>>{
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

    Ok(started_queue.name().to_string())
}

async fn init_receive_notification(channel: &Channel) -> Result<String, Box<dyn std::error::Error>>{
    channel
        .exchange_declare(
            "notificacoes",
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let notification_queue = channel
        .queue_declare(
            "", // random name
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    Ok(notification_queue.name().to_string())
}

async fn init_process_input(channel: &Channel) -> Result<(), Box<dyn std::error::Error>>{
    channel
        .queue_declare(
            "lance_realizado",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(())
}


// make and publish a bid
// async fn make_bid(auction_id: u32, client_id: u32, value: f64, bid_queue: &lapin::Queue, private_key: &RsaPrivateKey) -> Result<Bid, Box<dyn Error>> {
//     let content = format!("{}:{}:{}", auction_id, client_id, value).into_bytes();
//     let hashed = Sha256::digest(content);

//     // sign
//     let signature = private_key.sign(
//         Pkcs1v15Sign::new_unprefixed(),
//         &hashed,
//     )?;
//     let bid = Bid {
//         auction_id,
//         client_id,
//         value,
//         signature: general_purpose::STANDARD.encode(signature)
//     };

//     let payload = json!(bid).to_string();

//     channel.basic_publish(
//         "",
//         "lance_realizado",
//         BasicPublishOptions::default(),
//         payload.as_bytes(),
//         BasicProperties::default()
//     ).await?.await?;

//     Ok(())
// }