use lapin::options::{QueueDeclareOptions, QueueBindOptions, ExchangeDeclareOptions};
use lapin::{
    Channel, Connection, ConnectionProperties
};
use tokio::sync::mpsc;


use lapin::types::FieldTable;

use std::sync::Arc;
use crate::cli::Cli;

use std::{env, fs};

use tokio::{task::JoinHandle};
use rsa::{ RsaPrivateKey};
use rsa::pkcs1::{DecodeRsaPrivateKey};
use rsa::pkcs8::EncodePublicKey;

pub mod models;

use crate::models::*;

pub mod tasks;
use crate::tasks::{
    task_init_auction,
    task_receive_notification,
    task_make_bid,
    task_subscribe,
    task_cli
};
pub mod cli;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <client_id> <private_key_path>", args[0]);
        std::process::exit(1);
    }
    let client_id: u32 = args[1].parse()?;
    let private_key_path = &args[2];

    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");
    let conn = Arc::new(conn);

    let (started_queue_name, notification_queue_name) = init_rabbitmq_structs(conn.clone()).await?;

    // Load the private key from the specified file
    let pem = fs::read_to_string(private_key_path)?;
    let private_key = RsaPrivateKey::from_pkcs1_pem(&pem)?;

    let client = Client {
        id: client_id,
        subscribed_auctions: Vec::new(),
        private_key: private_key.clone(),
        public_key: private_key
            .to_public_key()
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)?
            .to_string(),
        notification_queue_name: notification_queue_name.clone(),
    };
    
    let handles = init_tasks(
        conn, 
        started_queue_name, 
        client
    );

        

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

fn init_tasks(
    conn: Arc<Connection>,
    started_queue_name: String,
    client: Client,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();

    let (make_bid_tx, make_bid_rx) = mpsc::channel::<Bid>(20);
    let (subscribe_tx, subscribe_rx) = mpsc::channel::<u32>(20);
    let (cli_print_tx, cli_print_rx) = mpsc::channel::<String>(100);
    let cli = Cli::new();

    let client_static = Arc::new(client.clone());

    handles.push(tokio::spawn(task_init_auction(
        conn.clone(),
        started_queue_name,
        client_static.clone(),
        cli_print_tx.clone()
    )));

    handles.push(tokio::spawn(task_receive_notification(
        conn.clone(),
        client_static.clone(),
        cli_print_tx.clone()
    )));

    handles.push(tokio::spawn(task_subscribe(
        conn.clone(),
        client,
        cli_print_tx,
        subscribe_rx,
    )));

    handles.push(tokio::spawn(task_make_bid(
        conn.clone(),
        client_static.clone(),
        make_bid_rx
    )));

    handles.push(tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(task_cli(make_bid_tx, subscribe_tx, cli_print_rx, cli))
    }));


    handles
}

async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<(String, String), Box<dyn std::error::Error>> {
    let channel = conn.create_channel().await?;

    let started_queue_name = init_leilao_iniciado(&channel).await?;
    let notification_queue_name = init_receive_notification(&channel).await?;
    init_process_input(&channel).await?;

    Ok((started_queue_name, notification_queue_name))
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
            lapin::ExchangeKind::Topic,
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


