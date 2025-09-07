use lapin::options::{QueueDeclareOptions, QueueBindOptions, ExchangeDeclareOptions};
use lapin::{
    Channel, Connection, ConnectionProperties
};
use tokio::sync::mpsc;


use lapin::types::FieldTable;

use std::sync::Arc;
use crate::cli::Cli;



use tokio::{sync::Mutex, task::JoinHandle};
use rsa::{ RsaPrivateKey};
use rsa::pkcs1::{DecodeRsaPrivateKey};
use rsa::pkcs8::EncodePublicKey;

pub mod models;

use crate::models::*;

pub mod tasks;
use crate::tasks::{
    task_init_auction,
    task_receive_notification,
    task_make_bid,
    task_subscribe,
    task_cli
};
pub mod cli;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Consumer connected to RabbitMQ!");
    let conn = Arc::new(conn);

    let (started_queue_name, notification_queue_name) = init_rabbitmq_structs(conn.clone()).await?;


    // create aaaa 1757163605 1757189999

    //let pem = fs::read_to_string("private_key.pem")?;
    let pem = "-----BEGIN RSA PRIVATE KEY-----
MIICWgIBAAKBgGmcA0BGDhveEY6+dmEo1lil2NpB0Y8NpXdpBUi5DZLbk9Sg/sTv
8z/AURr99a7MKAVFFngHTioOwxB5ruwhdvuFKKyqTnzYK2dv87WJ7GqqUda2rlhB
my4CCOXSS+YLqgdQYj4QesBDOC9ojdFaIPGIyp77J4iHAoICxN+y+Rn9AgMBAAEC
gYAarXFYzBmGSptuzogC1RkIPaTAxX2VQGI6/sl57F0Uauk1/hE9WEu/H+qdAegM
5r95TVF2som5MA9wWvyn43A1l+zjuHYZIZZTNguhbDZ+oFEWFxERzzqe1EF+DQ25
n4vV9H2Iww2KdbyC8RuabK/QRqeTBH+JNOw5Ng84Hkbg9QJBAKxSpaPsHeHR+r/Y
vqiuTqnzxp6WKkoJ5t+w6ZmCLisoAUpyjZcMKJfdditf1ypCL5CpbMxV/Dh0Wt8E
bvo2lwsCQQCc5EJqTCP9LVs9Pn4UoSQLoR9K39zu4sE0rbXyXsLx2dlb992ednCm
f89vZZ9hVaR5EUn+51s34QUxUnXdCZgXAkBeoniy4B29AUr6hraV/jvXG7hNKVyK
EowG9qojEpn2O18SGnzlodi9JfMaeOS6IWTrxg+o2+PKwSOSbGXh5Y7nAkA+ywTh
8nN9A0g/LOHdc9kvZl9V4l9UpSDa6qOly9OOZLigHIZww8q2ePUXCr9Nf6+CXS8W
fJZ/uOoRIYXW394lAkA89SZ9iU/6GkTG6z5hzENPfiuiPGV3Ytp+TgbDxprVhGrv
sdpoIwdgJDlOHWpRethjNiNx7FITofCcsqC/8Mp7
-----END RSA PRIVATE KEY-----";
    let private_key = RsaPrivateKey::from_pkcs1_pem(pem)?;


    let client = 
    Arc::new(Mutex::new(Client {
        id: 0,
        subscribed_auctions: Vec::new(),
        private_key: private_key.clone(),
        public_key: private_key.to_public_key().to_public_key_pem(rsa::pkcs8::LineEnding::LF)?.to_string(),
        notification_queue_name: notification_queue_name.clone(),
    }));
    
    let handles = init_tasks(conn, started_queue_name, client);

        

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

fn init_tasks(
    conn: Arc<Connection>,
    started_queue_name: String,
    client_mutex: Arc<Mutex<Client>>,
) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();

    let (make_bid_tx, make_bid_rx) = mpsc::channel::<Bid>(20);
    let (subscribe_tx, subscribe_rx) = mpsc::channel::<String>(20);
    let (cli_print_tx, cli_print_rx) = mpsc::channel::<String>(20);
    let cli = Cli::new();

    handles.push(tokio::spawn(task_init_auction(
        conn.clone(),
        started_queue_name,
        client_mutex.clone(),
        cli_print_tx.clone()
    )));

    handles.push(tokio::spawn(task_receive_notification(
        conn.clone(),
        client_mutex.clone(),
        cli_print_tx
    )));

    handles.push(tokio::spawn(task_subscribe(
        conn.clone(),
        client_mutex.clone(),
        subscribe_rx,
    )));

    handles.push(tokio::spawn(task_make_bid(
        conn.clone(),
        client_mutex.clone(),
        make_bid_rx
    )));

    handles.push(tokio::spawn(
        task_cli(make_bid_tx, subscribe_tx, cli_print_rx, cli)
    ));


    handles
}

async fn init_rabbitmq_structs(conn: Arc<Connection>) -> Result<(String, String), Box<dyn std::error::Error>> {
    let channel = conn.create_channel().await?;

    let started_queue_name = init_leilao_iniciado(&channel).await?;
    let notification_queue_name = init_receive_notification(&channel).await?;
    init_process_input(&channel).await?;

    Ok((started_queue_name, notification_queue_name))
}

async fn init_leilao_iniciado(channel: &Channel) -> Result<String, Box<dyn std::error::Error>>{
    channel
        .exchange_declare(
            "leilao_iniciado",
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let started_queue = channel
        .queue_declare(
            "", // random name
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    // bind the queue to the fanout exchange, to receive leilao_iniciado messages
    channel
        .queue_bind(
            started_queue.name().as_str(),
            "leilao_iniciado",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(started_queue.name().to_string())
}

async fn init_receive_notification(channel: &Channel) -> Result<String, Box<dyn std::error::Error>>{
    channel
        .exchange_declare(
            "notificacoes",
            lapin::ExchangeKind::Direct,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    
    let notification_queue = channel
        .queue_declare(
            "", // random name
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    


    Ok(notification_queue.name().to_string())
}

async fn init_process_input(channel: &Channel) -> Result<(), Box<dyn std::error::Error>>{
    channel
        .queue_declare(
            "lance_realizado",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;


    Ok(())
}


