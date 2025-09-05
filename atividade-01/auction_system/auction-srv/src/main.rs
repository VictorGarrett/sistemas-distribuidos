use lapin::{
    options::{ExchangeDeclareOptions, QueueDeclareOptions}, types::FieldTable, Connection, ConnectionProperties
};

use tokio::task::JoinHandle;
use std::{error::Error, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::mpsc;
use std::sync::Arc;

use crate::{cli::Cli, models::Auction, tasks::{task_cli, task_cron, task_publish_auction_finish, task_publish_auction_start}};

pub mod models;
pub mod tasks;
pub mod cli;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Arc::new(Connection::connect(addr, ConnectionProperties::default()).await?);


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
    let cli = Cli::new(Some(live_auctions.clone()));
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
            live_auctions, 
            new_auction_rx, 
            started_auction_tx, 
            finished_auction_tx
        )
    ));

    handles.push(tokio::spawn(
        task_cli(new_auction_tx, cli)
    ));

    handles
    
}

fn get_auctions() -> Vec<Auction> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    vec![
        Auction::new(
            1, 
            "1L de água de poça".to_string(), 
            now,
            now + 3 * 60 * 1000,        
        ),
        Auction::new(
            2, 
            "bituca de cigarro".to_string(), 
            now + 60 *1000,
            now + 6 * 60 * 1000,
        ),
    ]
}
