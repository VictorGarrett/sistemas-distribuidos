use std::sync::Arc;
use tokio::sync::Mutex;
use lapin::{
    options::{
        BasicConsumeOptions, 
        BasicPublishOptions, 
    }, 
    types::FieldTable, Channel, Connection
};
use futures_lite::stream::StreamExt;
use rsa::{
    pkcs8::DecodePublicKey, 
    RsaPublicKey, 
    Pkcs1v15Sign
};
use sha2::{Digest, Sha256};

use serde_json;

use crate::models::*;

use crate::client::Client;

/*==================================================== TASKS  ====================================================*/

pub async fn task_process_input(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
        "leilao_finalizado", 
        "bid-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction_id = u32::from_ne_bytes(delivery.data.as_slice().try_into().unwrap());
        let mut auctions = auctions.lock().await;
        if let Some(auction) = auctions.iter_mut().find(|a| a.id == auction_id){
            auction.is_active = false;
        }
        drop(auctions); //ensures lock is released before next iteration

        //If there is a bid for this auction, publish the winner bid
        let bids = bids.lock().await;
        if let Some(winning_bid) = bids
            .iter()
            .filter(|a| a.auction_id == auction_id)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
        {
            publish_winner_bid(&channel, winning_bid).await.unwrap();
        }
        drop(bids); //ensures lock is released before next iteration
            
    }
}

pub async fn task_receive_notification(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {

    let channel = conn.create_channel().await.unwrap();


    
    let mut consumer = channel.basic_consume(
        "lance_realizado_bid-srv", 
        "bid-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        
    }

}

pub async fn task_init_auction(
    conn: Arc<Connection>,
    started_queue_name: String,
    client_mutex: Arc<Mutex<Client>>
){
    let channel = conn.create_channel().await.unwrap();

    let client = client_mutex.lock().await;
    let mut consumer = channel.basic_consume(
            started_queue_name.as_str(), 
            &format!("client-started-consumer-{}", client.id),
            BasicConsumeOptions::default(), 
            FieldTable::default()
        )
        .await
        .unwrap();
    drop(client);

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction_id = u32::from_ne_bytes(delivery.data.as_slice().try_into().unwrap());
        let mut client = client_mutex.lock().await;
        
        let new_auction = Auction::new(auction_id, "something".to_string());
        client.subscribed_auctions.push(new_auction);
        drop(client); //ensures lock is released before next iteration

        
    }
}

/*==================================================== TASKS - END ====================================================*/


/*====================================================== AUX ====================================================== */

/*============================================= PUBLISH ============================================= */


async fn publish_validated_bid(
    channel: &Channel, 
    bid: &Bid
) -> Result<(), Box<dyn std::error::Error>> {
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

async fn publish_winner_bid(
    channel: &Channel, 
    bid: &Bid
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(bid)?;
    channel
        .basic_publish(
            "",
            "leilao_vencedor",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?
        .await?;
    Ok(())
}  

/*============================================= PUBLISH - END ============================================= */

/*============================================= BID VERIFICATION ============================================= */

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
    verify_bid(bid, public_key)
}

fn verify_bid(bid: &Bid, public_key: RsaPublicKey) -> bool {
    let content = format!("{}:{}:{}", bid.auction_id, bid.client_id, bid.value).into_bytes();
    let hashed = Sha256::digest(content);

    public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, bid.signature.as_bytes()).is_ok()
}

/*============================================= BID VERIFICATION - END ============================================= */
