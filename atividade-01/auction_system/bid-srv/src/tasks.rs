use std::sync::Arc;
use tokio::sync::Mutex;
use lapin::{options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueDeclareOptions}, types::FieldTable, Connection};
use futures_lite::stream::StreamExt;

use crate::models::*;


async fn task_end_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {

}

async fn task_validate_bid(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {

}

async fn task_init_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    conn: Arc<Connection>,
){
    let channel = conn.create_channel().await.unwrap();
    let _queue = channel
        .queue_declare(
            "leilao_iniciado", 
            QueueDeclareOptions::default(), 
            FieldTable::default(),
    ).await
    .expect("Failed to declare queue");

    let mut consumer = channel.basic_consume(
            "leilao_iniciado", 
            "bid-srv", 
            BasicConsumeOptions::default(), 
            FieldTable::default()
        ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction_id = u32::from_ne_bytes(delivery.data.as_slice().try_into().unwrap());
        let mut auctions = auctions.lock().await;
        
        let new_auction = Auction::new(auction_id, "something".to_string());
        auctions.push(new_auction);
        drop(auctions); //ensures lock is released before next iteration
    }
}