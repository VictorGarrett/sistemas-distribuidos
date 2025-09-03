use lapin::{
    Connection, ConnectionProperties,
};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};

pub mod models;
pub mod tasks;
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

    let auctions: Vec<Auction> = Vec::new();
    let bids: Vec<Bid> = Vec::new();
    let auctions = Arc::new(Mutex::new(auctions));
    let bids = Arc::new(Mutex::new(bids));

    let handles = init_tasks(auctions, bids, conn);

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

fn init_tasks(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
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
    )));

    handles
}
