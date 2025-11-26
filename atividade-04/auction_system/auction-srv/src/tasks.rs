use tokio::sync::{mpsc::{Receiver, Sender}};
use lapin::{options::BasicPublishOptions, BasicProperties, Connection};
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use tokio::sync::Mutex;

use std::{sync::Arc, time::{Duration, SystemTime}};

use shared::models::{
    Auction
};

#[derive(Deserialize)]
pub struct CreateAuctionRequest {
    pub start_timestamp: u128,
    pub end_timestamp: u128,
    pub item_name: String,
}

#[derive(Clone)]
struct AppState {
    new_auction_tx: Sender<Auction>,
    live_auctions: Arc<Mutex<Vec<Auction>>>,
    started_auctions: Arc<Mutex<Vec<Auction>>>,
    auction_counter: Arc<Mutex<u64>>
}

/// Task that runs the HTTP server and forwards auctions to the scheduler
pub async fn task_grpc_server() {

}

async fn create_auction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAuctionRequest>,
) -> Result<Json<Auction>, axum::http::StatusCode> {
    let mut counter = state.auction_counter.lock().await;

    let auction = Auction::new(
        (*counter) as u32, 
        req.item_name, 
        req.start_timestamp,
        req.end_timestamp
    );

    *counter = *counter + 1;

    println!("Request for auction: {:?}", auction);

    state.new_auction_tx
        .send(auction.clone())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(auction))
}


async fn list_auctions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Auction>>, axum::http::StatusCode> {
    let auctions = state.started_auctions.lock().await.clone();
    Ok(Json(auctions))
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
        let payload = auction.id.to_ne_bytes();
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
    live_auctions: Arc<Mutex<Vec<Auction>>>,
    started_auctions: Arc<Mutex<Vec<Auction>>>,
    mut new_auction_rx: Receiver<Auction>,
    mut started_auction_tx: Sender<Auction>,
    mut finished_auction_tx: Sender<Auction>
){
    let scheduled_auctions = live_auctions.lock().await;
    let mut finished_auctions: Vec<Auction> = Vec::with_capacity(scheduled_auctions.len());
    
    drop(scheduled_auctions);

    loop{

        let mut scheduled_auctions = live_auctions.lock().await;
        let mut started_auctions = started_auctions.lock().await;


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

        drop(scheduled_auctions);
        drop(started_auctions);

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
        println!("Starting auction: {:?}", auction);
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
