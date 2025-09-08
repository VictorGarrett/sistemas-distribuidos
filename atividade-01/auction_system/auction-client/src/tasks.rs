use std::sync::Arc;
use lapin::options::{QueueBindOptions};
use rsa::RsaPublicKey;
use tokio::sync::{mpsc::{Receiver, Sender}};
use rsa::pkcs8::DecodePublicKey;


use lapin::{
    options::{
        BasicConsumeOptions, 
        BasicPublishOptions, 
    }, 
    types::FieldTable, Channel, Connection
};
use futures_lite::stream::StreamExt;
use rsa::{
    Pkcs1v15Sign
};
use sha2::{Digest, Sha256};

use base64::{engine::general_purpose, Engine as _};
use serde_json;
use crate::models::{Bid, Client, Notification, NotificationType, Auction};
use crate::cli::Cli;



/*==================================================== TASKS  ====================================================*/


pub async fn task_cli(
    make_bid_tx: Sender<Bid>,
    subscribe_tx: Sender<u32>,
    cli_print_rx: Receiver<String>,
    cli: Cli
){
    cli.run(make_bid_tx, subscribe_tx, cli_print_rx).await.unwrap();

    eprintln!("Exiting CLI TASK");
}

pub async fn task_subscribe(
    conn: Arc<Connection>,
    mut client: Client,
    cli_print_tx: Sender<String>,
    mut subscribe_rx: Receiver<u32>,
) {
    let channel = conn.create_channel().await.unwrap();

    while let Some(auction_id) = subscribe_rx.recv().await {
        if let Some(_) = client.subscribed_auctions.iter().find(|&&a| a == auction_id){
            continue;
        }
        
        client.subscribed_auctions.push(auction_id);
        let routing_key = format!("leilao_{}", auction_id);
        
        let result = cli_print_tx
            .send(format!("[AUCTION] Subscribed to auction {}\n", auction_id))
            .await;

        if let Err(e) = result{
            eprintln!("Error!: {e}");
        }
        
        channel
            .queue_bind(
                client.notification_queue_name.as_str(),
                "notificacoes",
                routing_key.as_str(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .unwrap();
    }

}

pub async fn task_make_bid(
    conn: Arc<Connection>,
    client: Arc<Client>,
    mut make_bid_rx: Receiver<Bid>,
) {
    let channel = conn.create_channel().await.unwrap();

    while let Some(mut bid) = make_bid_rx.recv().await {

        bid.client_id = client.id;

        let content = format!("{}:{}:{}", bid.auction_id, bid.client_id, bid.value).into_bytes();
    
        let hashed = Sha256::digest(content);

        let thing: String = hashed.iter().map(|b| format!("{:02x}", b)).collect();
        println!("signing->{}:{}:{}\n{}", bid.auction_id, bid.client_id, bid.value, thing);

        // sign
        let signature = client.private_key.sign(
            Pkcs1v15Sign::new_unprefixed(),
            &hashed,
        ).unwrap();


        bid.signature = general_purpose::STANDARD.encode(signature);
        bid.public_key = client.public_key.clone();

        let public_key: RsaPublicKey = RsaPublicKey::from_public_key_pem(bid.public_key.as_str()).unwrap();
        println!("Signature valid: {}", public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &base64::engine::general_purpose::STANDARD.decode(&bid.signature).unwrap()).is_ok());

        publish_bid(&channel, &bid).await.unwrap();
    }
}

pub async fn task_receive_notification(
    conn: Arc<Connection>,
    client: Arc<Client>,
    cli_print_tx: Sender<String>,
) {

    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel
        .basic_consume(
            client.notification_queue_name.as_str(),
            &format!("client-notification-consumer-{}", client.id),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        // Deserialize the notification
        let notification: Notification = serde_json::from_slice(&delivery.data).unwrap();


        match notification.get_notification_type() {
            NotificationType::NewBid => {
                cli_print_tx
                    .send(format!(
                        "[NOTIFICATION] New bid: auction={} client={} value={}\n",
                        notification.get_auction_id(),
                        notification.get_client_id(),
                        notification.get_bid_value()
                    ))
                    .await
                    .unwrap();
            }
            NotificationType::AuctionWinner => {
                cli_print_tx
                    .send(format!(
                        "[NOTIFICATION] Auction winner: auction={} client={} value={}\n",
                        notification.get_auction_id(),
                        notification.get_client_id(),
                        notification.get_bid_value()
                    ))
                    .await
                    .unwrap();
            }
        }
    }

}

pub async fn task_init_auction(
    conn: Arc<Connection>,
    started_queue_name: String,
    client: Arc<Client>,
    cli_print_tx: Sender<String>,
){
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
            started_queue_name.as_str(), 
            &format!("client-started-consumer-{}", client.id),
            BasicConsumeOptions::default(), 
            FieldTable::default()
        )
        .await
        .unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction:Auction = serde_json::from_slice(&delivery.data).unwrap();
        
        if let Err(e) = cli_print_tx
            .send(format!(
                "[AUCTION] id={} item={} active={}\n",
                auction.id, auction.item, auction.status
            ))
            .await
        {
            eprintln!("Failed to send auction message to CLI: {}", e);
        }

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


/*============================================= BID VERIFICATION - END ============================================= */
