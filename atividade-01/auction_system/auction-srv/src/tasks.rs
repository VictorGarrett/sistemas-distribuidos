use serde::de::value::EnumAccessDeserializer;
use tokio::sync::{mpsc::{Receiver, Sender}};
use lapin::{options::BasicPublishOptions, types::FieldTable, BasicProperties, Connection};

use std::{sync::Arc, time::{Duration, SystemTime}};

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
    mut new_auction_rx: Receiver<Auction>,
    mut started_auction_tx: Sender<Auction>,
    mut finished_auction_tx: Sender<Auction>
){
    let mut scheduled_auctions = live_auctions;
    let mut finished_auctions: Vec<Auction> = Vec::with_capacity(scheduled_auctions.len());
    let mut started_auctions: Vec<Auction> = Vec::with_capacity(scheduled_auctions.len());
    loop{
        if let Ok(auction) = new_auction_rx.try_recv(){
            scheduled_auctions.push(auction)
        }

        let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

        if scheduled_auctions.iter().any(|auc| auc.start_timestamp <= now){
            start_auctions(
                &mut scheduled_auctions,
                &mut started_auctions,
                &mut started_auction_tx,
                now
            ).await;
        }

        if started_auctions.iter().any(|auc| auc.end_timestamp <= now){
            finish_auctions(
                &mut started_auctions,
                &mut finished_auctions,
                &mut finished_auction_tx,
                now
            ).await;
        }

        tokio::time::sleep(Duration::from_millis(300)).await;
    }

}

async fn start_auctions(
    scheduled_auctions: &mut Vec<Auction>,
    started_auctions: &mut Vec<Auction>,
    started_auction_tx: &mut Sender<Auction>,
    now: u128
){
    let auctions_to_start: Vec<Auction> = scheduled_auctions
        .iter()
        .filter(|a| a.start_timestamp <= now)
        .cloned()
        .collect();

    let mut auctions_to_start_cloned = auctions_to_start.clone();
    for auction in auctions_to_start{
        started_auction_tx
            .send(auction)
            .await
            .unwrap();
    }

    started_auctions.append(&mut auctions_to_start_cloned);
    *scheduled_auctions = scheduled_auctions
        .iter()
        .filter(|a| a.start_timestamp > now)
        .cloned()
        .collect();
}

async fn finish_auctions(
    started_auctions: &mut Vec<Auction>,
    ended_auctions: &mut Vec<Auction>,
    finished_auction_tx: &mut Sender<Auction>,
    now: u128
){
    let auctions_to_finish: Vec<Auction> = started_auctions
        .iter()
        .filter(|a| a.end_timestamp <= now)
        .cloned()
        .collect();
    
    let mut auctions_to_finish_cloned = auctions_to_finish.clone();
    for auction in auctions_to_finish{
        finished_auction_tx
            .send(auction)
            .await
            .unwrap();
    }

    ended_auctions.append(&mut auctions_to_finish_cloned);
    *started_auctions = started_auctions
        .iter()
        .filter(|a| a.end_timestamp > now)
        .cloned()
        .collect();

}