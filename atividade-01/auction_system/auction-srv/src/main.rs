use lapin::{
    options::{QueueDeclareOptions}, types::FieldTable, Connection, ConnectionProperties
};
use tokio::runtime::Runtime;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::auction::Auction;

pub mod auction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Runtime::new()?.block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;


    channel
        .exchange_declare(
            "leilao_iniciado",           // exchange name
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("declare exchange");

    channel.queue_declare(
        leilao_finalizado, 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    )
    .await?;

    
    let auctions = get_auctions();
    let mut auctions_ref: Vec<(&Auction, u128)>  = Vec::with_capacity(2 * auctions.len());
    auctions.iter().for_each(|a| {
        auctions_ref.push((a, a.start_timestamp));
        auctions_ref.push((a, a.end_timestamp));
    });
    auctions_ref.sort_by(|a, b| a.1.cmp(&b.1));


    for (auction, timestamp) in auctions_ref {
        let mut now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if timestamp > now {
            tokio::time::sleep(tokio::time::Duration::from_millis((timestamp - now) as u64)).await;
        }

        now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        if now > auction.end_timestamp {
            // publish to simple queue, this message goes to bid-srv only 
            channel.basic_publish(
                "", 
                "leilao_finalizado", 
                lapin::options::BasicPublishOptions::default(), 
                auction.id.to_ne_bytes().as_ref(), 
                lapin::BasicProperties::default()
            ).await?.await?;
        println!("Published to {}: {}", queue, auction.id); 
        } else {
            // publish to fanout exchange, this message goes to all clients 
            channel.basic_publish(
                "leilao_iniciado", 
                "", 
                lapin::options::BasicPublishOptions::default(), 
                auction.id.to_ne_bytes().as_ref(), 
                lapin::BasicProperties::default()
            ).await?.await?;
        println!("Published to {}: {}", queue, auction.id);
        };

        
    }

    Ok(())
}


fn get_auctions() -> Vec<Auction> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    vec![
        Auction::new(
            1, 
            "1L de água de poça".to_string(), 
            now,
            now + 15 * 60 * 1000,        
        ),
        Auction::new(
            2, 
            "bituca de cigarro".to_string(), 
            now + 60 * 1000,
            now + 20 * 60 * 1000,
        ),
    ]
}
