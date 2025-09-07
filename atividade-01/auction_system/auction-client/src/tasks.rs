use std::sync::Arc;
use tokio::sync::Mutex;
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
    subscribe_tx: Sender<String>,
    cli_print_rx: Receiver<String>,
    cli: Cli
){
    cli.run(make_bid_tx, subscribe_tx, cli_print_rx).await.unwrap();
}

pub async fn task_subscribe(
    conn: Arc<Connection>,
    client_mutex: Arc<Mutex<Client>>,
    mut subscribe_rx: Receiver<String>,
) {
    let channel = conn.create_channel().await.unwrap();

    while let Some(auction_id_str) = subscribe_rx.recv().await {
        if let Ok(auction_id) = auction_id_str.parse::<u32>() {
            let mut client = client_mutex.lock().await;
            client.subscribed_auctions.push(auction_id);

            channel
                .queue_bind(
                    &client.notification_queue_name,
                    "notificacoes",
                    &format!("leilao{}", auction_id),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();
            drop(client);
        } else {
            println!("Invalid auction id: {}", auction_id_str);
        }
    }

}

pub async fn task_make_bid(
    conn: Arc<Connection>,
    client_mutex: Arc<Mutex<Client>>,
    mut make_bid_rx: Receiver<Bid>,
) {
    let channel = conn.create_channel().await.unwrap();

    loop{
        while let Ok(mut bid) = make_bid_rx.try_recv() {

            

            let client = client_mutex.lock().await;

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

            drop(client);

            let public_key: RsaPublicKey = RsaPublicKey::from_public_key_pem(bid.public_key.as_str()).unwrap();
            println!("Signature valid: {}", public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &base64::engine::general_purpose::STANDARD.decode(&bid.signature).unwrap()).is_ok());


            publish_bid(&channel, &bid).await.unwrap();
        }
    }
}

pub async fn task_receive_notification(
    conn: Arc<Connection>,
    client_mutex: Arc<Mutex<Client>>,
    cli_print_tx: Sender<String>,
) {

    let channel = conn.create_channel().await.unwrap();
    let client = client_mutex.lock().await;

    let mut consumer = channel
        .basic_consume(
            client.notification_queue_name.as_str(),
            &format!("client-notification-consumer-{}", client.id),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();
    drop(client);

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        // Deserialize the notification
        let notification: Notification = serde_json::from_slice(&delivery.data).unwrap();


        match notification.get_notification_type() {
            NotificationType::NewBid => {
                cli_print_tx
                    .send(format!(
                        "[NOTIFICATION] New bid: auction={} client={} value={}",
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
                        "[NOTIFICATION] Auction winner: auction={} client={} value={}",
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
    client_mutex: Arc<Mutex<Client>>,
    cli_print_tx: Sender<String>,
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
        println!("[AUCTION STARTED] Received auction data: {:?}", delivery.data);

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
