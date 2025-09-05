use std::sync::{mpsc::{Receiver, Sender}, Arc};
use lapin::Connection;

use crate::models::Auction;


pub fn task_cli(
    new_auction_tx: Sender<Auction>
){

}

pub async fn task_publish_auction_start(
    conn: Arc<Connection>,
    started_auction_rx: Receiver<Auction>
){

}

pub async fn task_publish_auction_finish(
    conn: Arc<Connection>,
    finished_auction_rx: Receiver<Auction>
){

}

pub async fn task_cron(
    live_auctions: Vec<Auction>,
    new_auction_rx: Receiver<Auction>,
    started_auction_tx: Sender<Auction>,
    finished_auction_tx: Sender<Auction>
){

}