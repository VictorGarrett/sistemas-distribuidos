use lapin::{
    options::{BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions}, types::FieldTable, Channel, Connect, Connection, ConnectionProperties
};
use futures_lite::stream::StreamExt;

use std::{error::Error, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");

    

    Ok(())
}

async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<(), Box<dyn Error>>{
    let channel = conn.create_channel().await?;
    init_lance_validado(&channel).await?;
    init_leilao_vencedor(&channel).await?;
    init_notificacoes(&channel).await?;


    Ok(())
}

async fn init_lance_validado(channel: &Channel) -> Result<(), Box<dyn Error>>{
    channel.queue_declare(
        "lance_validado", 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    ).await?;

    channel.queue_bind(
        "lance_validado", 
        "", 
        "", 
        QueueBindOptions::default(), 
        FieldTable::default()
    ).await?;

    Ok(())
}

async fn init_leilao_vencedor(channel: &Channel) -> Result<(), Box<dyn Error>>{
    channel.queue_declare(
        "leilao_vencedor", 
        QueueDeclareOptions::default(), 
        FieldTable::default()
    ).await?;

    channel.queue_bind(
        "leilao_vencedor", 
        "", 
        "", 
        QueueBindOptions::default(), 
        FieldTable::default()
    ).await?;
    Ok(())
}

async fn init_notificacoes(channel: &Channel) -> Result<(), Box<dyn Error>>{
    channel.exchange_declare(
        "notificacoes", 
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await?;

    
    Ok(())
}
