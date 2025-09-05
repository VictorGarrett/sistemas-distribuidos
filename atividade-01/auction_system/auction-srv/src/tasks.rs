use tokio::sync::{mpsc::{Receiver, Sender}};
use lapin::{options::BasicPublishOptions, types::FieldTable, BasicProperties, Connection};

use std::sync::Arc;

use crate::models::Auction;
use crate::cli::Cli;

pub fn task_cli(
    new_auction_tx: Sender<Auction>,
    cli: Cli
){
    cli.run(new_auction_tx);
}

pub async fn task_publish_auction_start(
    conn: Arc<Connection>,
    mut started_auction_rx: Receiver<Auction>
){
    let channel = conn.create_channel().await.unwrap();

    while let Some(auction) = started_auction_rx.recv().await{
        let payload = serde_json::to_vec(&auction).unwrap();
        channel
            .basic_publish(
                "leilao_iniciado",
                "", 
                BasicPublishOptions::default(),
                payload.as_slice(),
                BasicProperties::default()
            ).await.unwrap();
    }
}

pub async fn task_publish_auction_finish(
    conn: Arc<Connection>,
    mut finished_auction_rx: Receiver<Auction>
){
    let channel = conn.create_channel().await.unwrap();

    while let Some(auction) = finished_auction_rx.recv().await{
        let payload = serde_json::to_vec(&auction).unwrap();
        channel
            .basic_publish(
                "",
                "leilao_finalizado",
                BasicPublishOptions::default(),
                payload.as_slice(),
                BasicProperties::default()
            ).await.unwrap();
    }
}

pub async fn task_cron(
    live_auctions: Vec<Auction>,
    new_auction_rx: Receiver<Auction>,
    started_auction_tx: Sender<Auction>,
    finished_auction_tx: Sender<Auction>
){

}