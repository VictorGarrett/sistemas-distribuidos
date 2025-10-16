use lapin::{
    options::{
        ExchangeDeclareOptions,
        QueueDeclareOptions}, 
        types::FieldTable, 
        Channel,
        Connection, 
        ConnectionProperties
};

use tokio::task::JoinHandle;
use std::{error::Error, sync::Arc};

mod tasks;

use crate::tasks::{
    task_notify_bid,
    task_notify_winner
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Arc::new(Connection::connect(addr, ConnectionProperties::default()).await?);
    println!("Consumer connected to RabbitMQ!");

    if let Err(e) = init_rabbitmq_structs(conn.clone()).await{
        println!("Failed to start RabbitMQ Structures. Exiting...");
        println!("Err: {e}");
        return Ok(());
    }

    let handles = init_tasks(conn);

    for handle in handles{
        handle.await?;
    }

    Ok(())
}

fn init_tasks(conn: Arc<Connection>) -> Vec<JoinHandle<()>>{
    let mut handles = Vec::with_capacity(2);
    
    handles.push(tokio::spawn(task_notify_bid(conn.clone())));
    handles.push(tokio::spawn(task_notify_winner(conn.clone())));

    handles
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

    Ok(())
}

async fn init_leilao_vencedor(channel: &Channel) -> Result<(), Box<dyn Error>>{
    channel.queue_declare(
        "leilao_vencedor", 
        QueueDeclareOptions::default(), 
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
