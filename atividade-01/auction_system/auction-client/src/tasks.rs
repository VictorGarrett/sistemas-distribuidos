use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use lapin::options::{QueueBindOptions};


use lapin::{
    options::{
        BasicConsumeOptions, 
        BasicPublishOptions, 
    }, 
    types::FieldTable, Channel, Connection
};
use futures_lite::stream::StreamExt;
use rsa::{
    RsaPrivateKey,
    Pkcs1v15Sign
};
use sha2::{Digest, Sha256};

use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType, enable_raw_mode},
};
use std::io::{self, Write};
use base64::{engine::general_purpose, Engine as _};

use serde_json;

use crate::models::*;

/*==================================================== TASKS  ====================================================*/

pub async fn task_process_input(
    conn: Arc<Connection>,
    client_mutex: Arc<Mutex<Client>>,
    prompt: Arc<Mutex<String>>,
) {
    let channel = conn.create_channel().await.unwrap();

    enable_raw_mode().unwrap();
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    loop {
        {
            let p = prompt.lock().await;
            print!("{}", *p);
            io::stdout().flush().unwrap();
        }

        if let Ok(Some(line)) = reader.next_line().await {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "subscribe" if parts.len() == 2 => {
                    if let Ok(id) = parts[1].parse::<u32>() {
                        let mut client = client_mutex.lock().await;
                        client.subscribed_auctions.push(id);

                        channel
                            .queue_bind(
                                &client.notification_queue_name,
                                "notification",
                                &format!("leilao{}", id),
                                QueueBindOptions::default(),
                                FieldTable::default(),
                            )
                            .await
                            .unwrap();
                        drop(client);
                    } else {
                        // Safe printing without breaking the prompt
                        let p = prompt.lock().await;
                        execute!(
                            io::stdout(),
                            cursor::SavePosition,
                            Clear(ClearType::CurrentLine),
                        )
                        .unwrap();
                        println!("Invalid auction id");
                        print!("{}", *p);
                        io::stdout().flush().unwrap();
                        execute!(io::stdout(), cursor::RestorePosition).unwrap();
                    }
                }
                "bid" if parts.len() == 3 => {
                    if let (Ok(auction_id), Ok(value)) =
                        (parts[1].parse::<u32>(), parts[2].parse::<f64>())
                    {
                        let client = client_mutex.lock().await;
                        let bid = make_bid(
                            auction_id,
                            client.id,
                            value,
                            &client.private_key,
                            client.public_key.clone(),
                        )
                        .await;
                        drop(client);
                        publish_bid(&channel, &bid).await;
                    } else {
                        let p = prompt.lock().await;
                        execute!(
                            io::stdout(),
                            cursor::SavePosition,
                            Clear(ClearType::CurrentLine),
                        )
                        .unwrap();
                        println!("Invalid bid format");
                        print!("{}", *p);
                        io::stdout().flush().unwrap();
                        execute!(io::stdout(), cursor::RestorePosition).unwrap();
                    }
                }
                _ => {
                    let p = prompt.lock().await;
                    execute!(
                        io::stdout(),
                        cursor::SavePosition,
                        Clear(ClearType::CurrentLine),
                    )
                    .unwrap();
                    println!("Unknown command. Use: subscribe <id> or bid <id> <value>");
                    print!("{}", *p);
                    io::stdout().flush().unwrap();
                    execute!(io::stdout(), cursor::RestorePosition).unwrap();
                }
            }
        }
    }
}

pub async fn task_receive_notification(
    conn: Arc<Connection>,
    client_mutex: Arc<Mutex<Client>>,
    prompt: Arc<Mutex<String>>
) {

    let channel = conn.create_channel().await.unwrap();
    let client = client_mutex.lock().await;

    let mut consumer = channel
        .basic_consume(
            "",
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

        // Lock the prompt to safely redraw
        let p = prompt.lock().await;

        // Save cursor, clear current line, print notification, restore prompt
        execute!(
            io::stdout(),
            cursor::SavePosition,
            Clear(ClearType::CurrentLine),
        )
        .unwrap();

        match notification.get_notification_type() {
            NotificationType::NewBid => {
                println!(
                    "[NOTIFICATION] New bid: auction={} client={} value={}",
                    notification.get_auction_id(),
                    notification.get_client_id(),
                    notification.get_bid_value()
                );
            }
            NotificationType::AuctionWinner => {
                println!(
                    "[NOTIFICATION] Auction winner: auction={} client={} value={}",
                    notification.get_auction_id(),
                    notification.get_client_id(),
                    notification.get_bid_value()
                );
            }
        }

        // Reprint prompt
        print!("{}", *p);
        io::stdout().flush().unwrap();
        execute!(io::stdout(), cursor::RestorePosition).unwrap();
    }

}

pub async fn task_init_auction(
    conn: Arc<Connection>,
    started_queue_name: String,
    client_mutex: Arc<Mutex<Client>>,
    prompt: Arc<Mutex<String>>
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


        let auction:Auction = serde_json::from_slice(&delivery.data).unwrap();
        
        let p: tokio::sync::MutexGuard<'_, String> = prompt.lock().await;
        execute!(
            io::stdout(),
            cursor::SavePosition,
            Clear(ClearType::CurrentLine),
        )
        .unwrap();

        println!(
            "[AUCTION] id={} item={} active={}",
            auction.id, auction.item, auction.is_active
        );

        print!("{}", *p);
        io::stdout().flush().unwrap();
        execute!(io::stdout(), cursor::RestorePosition).unwrap();


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

/*============================================= BID MAKING ============================================= */
// make and publish a bid
async fn make_bid(auction_id: u32, client_id: u32, value: f64, private_key: &RsaPrivateKey, public_key: String) -> Bid {
    
    let content = format!("{}:{}:{}", auction_id, client_id, value).into_bytes();
    
    let hashed = Sha256::digest(content);

    // sign
    let signature = private_key.sign(
        Pkcs1v15Sign::new_unprefixed(),
        &hashed,
    ).unwrap();
    
    return Bid {
        auction_id,
        client_id,
        value,
        signature: general_purpose::STANDARD.encode(signature),
        public_key: public_key,
        valid: false
    };

}

/*============================================= BID VERIFICATION - END ============================================= */
