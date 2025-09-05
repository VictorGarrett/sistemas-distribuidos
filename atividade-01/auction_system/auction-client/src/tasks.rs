use std::{io, sync::{mpsc::{Receiver, Sender}, Arc}};
use tokio::sync::Mutex;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions 
    }, 
    types::FieldTable, Channel, Connection
};
use futures_lite::stream::StreamExt;
use rsa::{
    RsaPublicKey, 
    Pkcs1v15Sign
};
use sha2::{Digest, Sha256};

use serde_json;

use crate::{cli::Cli, models::*};
use crate::command::{Command, parse_cmd};

/*==================================================== TASKS  ====================================================*/

pub fn task_cli(
    sender: Sender<Command>
) {
    let mut buffer = String::with_capacity(100);
    let stdin = io::stdin();

    while let Ok(read_bytes) = stdin.read_line(&mut buffer){
        match parse_cmd(buffer[..read_bytes].as_ref()){
            Some(cmd) => sender.send(cmd).unwrap(),
            None => println!("Invalid command.")
        }
    }
}

pub async fn task_receive_notification(
    conn: Arc<Connection>,
    notification_mq: String
) {

    let channel = conn.create_channel().await.unwrap();
    let mut consumer = channel.basic_consume(
        notification_mq.as_str(), 
        "bid-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();
    }
}

pub async fn task_publish_cmd(
    conn: Arc<Connection>,
    receiver: Receiver<Command>
){
    let channel = conn.create_channel().await.unwrap();

    while let Ok(cmd) = receiver.recv(){
        let Command::PlaceBid(auction_id, bid_value) = cmd;
        let bid = Bid::new(auction_id, 0, bid_value);
        publish_bid(&channel, &bid).await.unwrap();
    }

}

pub async fn task_leilao_iniciado(
    conn: Arc<Connection>,
    leilao_iniciado_mq: String,
    client_tag: String
){
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
        leilao_iniciado_mq.as_str(), 
        client_tag.as_str(), 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await{
        let delivery = delivery.unwrap();
        delivery.ack(BasicAckOptions::default()).await.unwrap();

        let auction_id = u32::from_ne_bytes(delivery.data.as_slice().try_into().unwrap());
        println!("New auction started!\nAuction ID: {auction_id}\n>");
    }
}

/*==================================================== TASKS - END ====================================================*/


/*====================================================== AUX ====================================================== */

/*============================================= PUBLISH ============================================= */


async fn publish_bid(
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
