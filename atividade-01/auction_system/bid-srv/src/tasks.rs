use std::sync::Arc;
use tokio::sync::Mutex;
use lapin::{options::{BasicConsumeOptions, BasicPublishOptions, ExchangeBindOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, protocol::queue, types::FieldTable, Channel, Connection};
use futures_lite::stream::StreamExt;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};

use serde_json;

use crate::models::*;


pub async fn task_end_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {

}

pub async fn task_validate_bid(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {

    let channel = task_validate_bid_setup(conn.clone()).await.unwrap();

    let mut consumer = channel.basic_consume(
        "lance_realizado_bid-srv", 
        "bid-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let bid: Bid = serde_json::from_slice(&delivery.data).unwrap();
        let public_key = RsaPublicKey::from_public_key_pem(bid.public_key.as_str()).unwrap();
        
        let bid_is_valid = is_bid_valid(
            &bid,
            &auctions,
            &bids,
            public_key
        );
        if bid_is_valid {
            let mut bids = bids.lock().await;
            bids.push(bid.clone());
            drop(bids); //ensures lock is released before next iteration

            publish_validated_bid(&channel, &bid).await.unwrap();
        } else {
            println!("Invalid bid signature");
        }
    }

}

pub async fn task_init_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    conn: Arc<Connection>,
){
    let (channel, queue_name) = task_init_auction_setup(conn.clone()).await.unwrap();
    let mut consumer = channel.basic_consume(
            queue_name.as_str(), 
            "bid-srv",
            BasicConsumeOptions::default(), 
            FieldTable::default()
        )
        .await
        .unwrap();

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


/*============================================ AUX ==================================================== */

fn is_bid_valid(
    bid: &Bid,
    auctions: &Arc<Mutex<Vec<Auction>>>,
    bids: &Arc<Mutex<Vec<Bid>>>,
    public_key: RsaPublicKey
) -> bool {
    let auctions = auctions.blocking_lock();
    let auction_opt = auctions.iter().find(|a| a.id == bid.auction_id && a.is_active);

    //Auction has ended or does not exist
    if auction_opt.is_none() {
        return false;
    }

    let bids = bids.blocking_lock();
    let highest_bid_opt = bids
        .iter()
        .filter(|b| b.auction_id == bid.auction_id)
        .max_by(|a, b| a.value.partial_cmp(&b.value)
        .unwrap());

    if let Some(highest_bid) = highest_bid_opt {
        if bid.value <= highest_bid.value {
            return false;
        }
    }
    true
}

fn verify_bid(bid: &Bid, public_key: RsaPublicKey) -> bool {

    let content = format!("{}:{}:{}", bid.auction_id, bid.client_id, bid.value).into_bytes();
    
    let hashed = Sha256::digest(content);

    match public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &signature) {
        Ok(_) => return true,
        Err(e) => return false,
    }
}

async fn publish_validated_bid(channel: &Channel, bid: &Bid) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(bid)?;
    channel
        .basic_publish(
            "",
            "lance_realizado",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?
        .await?;
    Ok(())
}

async fn task_validate_bid_setup(conn: Arc<Connection>) -> Result<Channel, Box<dyn std::error::Error>>{
    let channel = conn.create_channel().await?;
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

    channel.exchange_declare(
        "lance_realizado", 
        lapin::ExchangeKind::Topic, 
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    )
    .await?;

    Ok(channel)
}

async fn task_init_auction_setup(conn: Arc<Connection>) -> Result<(Channel, String), Box<dyn std::error::Error>>{
    let channel = conn.create_channel().await?;
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

    Ok((channel, fo_queue.name().to_string()))
}