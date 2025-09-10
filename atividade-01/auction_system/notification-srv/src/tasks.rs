use futures_lite::StreamExt;
use lapin::{options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions}, types::FieldTable, BasicProperties, Channel, Connection};
use std::{error::Error, sync::Arc};

use shared::models::{Bid, Notification, NotificationType};



pub async fn task_notify_bid(conn: Arc<Connection>){
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
        "lance_validado", 
        "notification-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    )
    .await.unwrap();

    while let Some(delivery) = consumer.next().await{
        let delivery = delivery.unwrap();
        delivery.ack(BasicAckOptions::default()).await.unwrap();
        let bid: Bid = serde_json::from_slice(&delivery.data).unwrap();

        publish_to_notification_exc(
            &channel, 
            &bid,
            NotificationType::NewBid
        ).await.unwrap();
    }
}

pub async fn task_notify_winner(conn: Arc<Connection>){
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
        "leilao_vencedor", 
        "notification-srv",
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await{
        let delivery = delivery.unwrap();
        delivery.ack(BasicAckOptions::default()).await.unwrap();

        let bid: Bid = serde_json::from_slice(&delivery.data).unwrap();

        publish_to_notification_exc(
            &channel, 
            &bid, 
            NotificationType::AuctionWinner
        ).await.unwrap();


    }
}

/*====================================================== AUX ====================================================== */

/*============================================= PUBLISH ============================================= */

async fn publish_to_notification_exc(channel: &Channel, bid: &Bid, notification_type: NotificationType) -> Result<(), Box<dyn Error>>{
    let notification = Notification::from_bid(
        bid,
        notification_type
    );
    let routing_key: String = build_routing_key(notification.get_auction_id());
    let payload = serde_json::to_vec(&notification).unwrap();

    channel.basic_publish(
        "notificacoes",
        routing_key.as_str(),
        BasicPublishOptions::default(), 
        &payload, 
        BasicProperties::default()
    ).await?.await?;

    println!("Published notification to notificacoes using routing key '{}'", routing_key);
    dbg!(&notification);

    Ok(())
}

/*============================================= PUBLISH - END ============================================= */

fn build_routing_key(auction_id: u32) -> String{
    format!("leilao_{auction_id}")
}