use lapin::{
    Connection, ConnectionProperties,
};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};
use lapin::options::{QueueDeclareOptions, QueueBindOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;


pub mod models;
pub mod tasks;
pub mod constants;

use crate::models::*;
use crate::tasks::{
    task_validate_bid,
    task_end_auction,
    task_init_auction
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");
    let conn = Arc::new(conn);

    let init_structs_res = init_rabbitmq_structs(conn.clone()).await;
    if init_structs_res.is_err() {
        println!("Error initializing RabbitMQ structures: {}", init_structs_res.as_ref().unwrap_err());
        return Err(init_structs_res.unwrap_err());
    }

    let fo_queue_name = init_structs_res.unwrap();
    println!("Initialized RabbitMQ structures");

    let auctions: Vec<Auction> = Vec::new();
    let bids: Vec<Bid> = Vec::new();
    let auctions = Arc::new(Mutex::new(auctions));
    let bids = Arc::new(Mutex::new(bids));

    let handles = init_tasks(auctions, bids, conn, fo_queue_name);
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

fn init_tasks(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
    fo_queue_name: String,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();

    handles.push(tokio::spawn(task_validate_bid(
        auctions.clone(),
        bids.clone(),
        conn.clone(),
    )));

    handles.push(tokio::spawn(task_end_auction(
        auctions.clone(),
        bids.clone(),
        conn.clone(),
    )));

    handles.push(tokio::spawn(task_init_auction(
        auctions.clone(),
        conn.clone(),
        fo_queue_name
    )));

    handles
}


async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<String, Box<dyn std::error::Error>> {
    let channel = conn.create_channel().await?;

    channel.exchange_declare(
        "lance_realizado", 
        lapin::ExchangeKind::Topic, 
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    )
    .await?;

    channel.queue_declare(
        "lance_realizado_bid-srv",
        QueueDeclareOptions::default(), 
        FieldTable::default(),
    ).await?;

    channel.queue_bind(
        "lance_realizado_bid-srv",
        "leilao_realizado", 
        "",
        QueueBindOptions::default(), 
        FieldTable::default()
    )
    .await?;

    let fo_queue = channel.queue_declare(
        "", 
        QueueDeclareOptions::default(), 
        FieldTable::default(),
    )
    .await?;

    channel.queue_bind(
        fo_queue.name().as_str(), 
        "leilao_iniciado", 
        "", 
        QueueBindOptions::default(), 
        FieldTable::default()
    )
    .await?;

    let _leilao_finalizado_mq = channel.queue_declare(
        "leilao_finalizado",
        QueueDeclareOptions::default(), 
        FieldTable::default(),
    ).await?;

    channel.queue_bind(
        "leilao_finalizado",
        "",
        "",
        QueueBindOptions::default(), 
        FieldTable::default()
    )
    .await?;

    let _leilao_vencedor_mq = channel.queue_declare(
        "leilao_vencedor",
        QueueDeclareOptions::default(), 
        FieldTable::default(),
    ).await?;

    channel.queue_bind(
        "leilao_vencedor",
        "",
        "",
        QueueBindOptions::default(), 
        FieldTable::default()
    )
    .await?;

    Ok(fo_queue.name().to_string())
}
