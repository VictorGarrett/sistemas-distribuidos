use lapin::{
    options::{ExchangeDeclareOptions, QueueDeclareOptions}, types::FieldTable, Connection, ConnectionProperties
};
use tokio::sync::Mutex;
use tokio::task::{JoinHandle};
use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::env;

use crate::{tasks::{task_rest_api, task_cron, task_publish_auction_finish, task_publish_auction_start}};

use shared::models::Auction;

pub mod models;
pub mod tasks;




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = env::var("RMQ_URL").unwrap_or("amqp://guest:guest@127.0.0.1:5672/%2f".to_string());
    let conn = Arc::new(Connection::connect(addr.as_str(), ConnectionProperties::default()).await?);


    init_rabbitmq_structs(conn.clone()).await?;
    let live_auctions = get_auctions();

    let handles = init_tasks(conn, live_auctions);

    for handle in handles{
        handle.await?;
    }

    Ok(())    
}

async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<(), Box<dyn Error>>{
    let channel = conn.create_channel().await?;
    channel.exchange_declare(
        "leilao_iniciado",           // exchange name
        lapin::ExchangeKind::Fanout,
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    ).await?;

    channel.queue_declare(
        "leilao_finalizado", 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    )
    .await?;

    Ok(())
}

fn init_tasks(
    conn: Arc<Connection>,
    live_auctions: Vec<Auction>
) -> Vec<JoinHandle<()>>{
    let mut handles = Vec::new();

    let (started_auction_tx, started_auction_rx) = mpsc::channel::<Auction>(20);
    let (finished_auction_tx, finished_auction_rx) = mpsc::channel::<Auction>(20);
    let (new_auction_tx, new_auction_rx) = mpsc::channel::<Auction>(20);

    let started_auctions: Vec<Auction> = Vec::with_capacity(live_auctions.len());
    let live_auctions = Arc::new(Mutex::new(live_auctions));
    let started_auctions = Arc::new(Mutex::new(started_auctions));



    handles.push(tokio::spawn(
        task_publish_auction_start(
            conn.clone(),
            started_auction_rx
        )
    ));

    handles.push(tokio::spawn(
        task_publish_auction_finish(
            conn.clone(), 
            finished_auction_rx
        )
    ));

    handles.push(tokio::spawn(
        task_cron(
            Arc::clone(&live_auctions), 
            Arc::clone(&started_auctions), 
            new_auction_rx, 
            started_auction_tx,
            finished_auction_tx
        )
    ));
    
    let rest_url = env::var("BASE_URL").unwrap_or("127.0.0.1".to_string());
    let rest_port = env::var("PORT").unwrap_or("8080".to_string());
    let rest_addr: String = rest_url + ":" + rest_port.as_ref();
    handles.push(tokio::spawn(
        task_rest_api(
            new_auction_tx,
            Arc::clone(&live_auctions),
            Arc::clone(&started_auctions),
            rest_addr,
        )
    ));

    handles
    
}

fn get_auctions() -> Vec<Auction> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    vec![
        Auction::new(
            0, 
            "1L de água de poça".to_string(), 
            now,
            now + 3 * 60 * 1000,        
        ),
        Auction::new(
            1, 
            "bituca de cigarro".to_string(), 
            now + 60 *1000,
            now + 6 * 60 * 1000,
        ),
    ]
}
