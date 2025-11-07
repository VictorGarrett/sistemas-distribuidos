use std::sync::Arc;
use tokio::sync::Mutex;
use lapin::{
    options::{
        BasicConsumeOptions, 
        BasicPublishOptions, 
    }, 
    types::FieldTable, Channel, Connection
};
use futures_lite::stream::StreamExt;
use rsa::{
    pkcs8::DecodePublicKey, 
    RsaPublicKey, 
    Pkcs1v15Sign
};
use axum::{
    extract::State,
    routing::post,
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use base64::engine::general_purpose;
use base64::Engine;
use std::{fs, path::Path};
use serde_json;

use shared::models::{
    Auction,
    Bid
};

/*==================================================== TASKS  ====================================================*/

pub async fn task_end_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {
    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel.basic_consume(
        "leilao_finalizado", 
        "bid-srv", 
        BasicConsumeOptions::default(), 
        FieldTable::default()
    ).await.unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction_id = u32::from_ne_bytes(delivery.data.as_slice().try_into().unwrap());
        println!("Received delivery on leilao_finalizado: {auction_id}");
        let mut auctions = auctions.lock().await;
        if let Some(auction) = auctions.iter_mut().find(|a| a.id == auction_id){
            auction.status = false;
        }
        drop(auctions); //ensures lock is released before next iteration

        //If there is a bid for this auction, publish the winner bid
        let bids = bids.lock().await;
        if let Some(winning_bid) = bids
            .iter()
            .filter(|a| a.auction_id == auction_id)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
        {
            publish_winner_bid(&channel, winning_bid).await.unwrap();
        }
        else{
            println!("No bid found for auction {auction_id}");
        }
        drop(bids); //ensures lock is released before next iteration
            
    }
}

#[derive(Clone)]
struct AppState {
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
    public_keys: Vec<Option<RsaPublicKey>>,
}

pub async fn task_validate_bid(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
) {


    let public_keys = load_public_keys_vec("bid-srv/keys").unwrap();

    let app_state = Arc::new(AppState {
        auctions: auctions,
        bids: bids,
        conn: conn,
        public_keys: public_keys,
    });

    let app = Router::new()
        .route("/bid", post(make_bid_handler))
        .with_state(app_state);

    let addr: std::net::SocketAddr = "127.0.0.1:8081".parse().unwrap();
    println!("REST API listening on {}", addr);

    // 1. Bind a tokio::net::TcpListener
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // 2. Use axum::serve to run the server
    axum::serve(listener, app.into_make_service()) // 'app' is your axum Router
        .await
        .unwrap();

    

}

async fn make_bid_handler(
    State(state): State<Arc<AppState>>, 
    Json(bid): Json<Bid>,          
) -> Response {                    
    
    println!("Received bid via HTTP POST");
    dbg!(&bid);

    let public_key = match state.public_keys.get(bid.client_id as usize) {
        Some(Some(key)) => key.clone(),
        _ => {
            println!("Invalid client ID or key not found for: {}", bid.client_id);
            return (StatusCode::BAD_REQUEST, "Invalid client ID or key not found").into_response();
        }
    };

    let bid_is_valid = is_bid_valid(
        &bid,
        &state.auctions,
        &state.bids,
        public_key
    ).await;


    if bid_is_valid {
        // Add to bids vector
        let mut bids_lock = state.bids.lock().await;
        bids_lock.push(bid.clone());
        drop(bids_lock); // Release lock

        let channel = match state.conn.create_channel().await {
            Ok(channel) => channel,
            Err(e) => {
                eprintln!("Failed to create RabbitMQ channel: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to queue").into_response();
            }
        };

        if let Err(e) = publish_validated_bid(&channel, &bid).await {
            eprintln!("Failed to publish validated bid: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Bid saved, but failed to publish").into_response();
        }

        (StatusCode::CREATED, "Bid accepted").into_response()
    } else {
        println!("Bid was deemed invalid");
        (StatusCode::BAD_REQUEST, "Bid was deemed invalid (e.g., signature failure)").into_response()
    }
}

pub async fn task_init_auction(
    auctions: Arc<Mutex<Vec<Auction>>>,
    conn: Arc<Connection>,
    fo_queue_name: String,
){
    let channel = conn.create_channel().await.unwrap();
    let mut consumer = channel.basic_consume(
            fo_queue_name.as_str(), 
            "bid-srv",
            BasicConsumeOptions::default(), 
            FieldTable::default()
        )
        .await
        .unwrap();

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.unwrap();
        delivery.ack(Default::default()).await.unwrap();

        let auction: Auction = serde_json::from_slice(delivery.data.as_ref()).unwrap();
        println!("Received delivery on leilao_iniciado: {}", auction.id);

        let mut auctions = auctions.lock().await;
        
        auctions.push(auction);
        drop(auctions); //ensures lock is released before next iteration
    }
}

/*==================================================== TASKS - END ====================================================*/


/*====================================================== AUX ====================================================== */
fn load_public_keys_vec<P: AsRef<Path>>(folder: P) -> std::io::Result<Vec<Option<RsaPublicKey>>> {
    let mut keys = Vec::new();

    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(id_str) = filename.strip_prefix("client_") {
                if let Ok(id) = id_str.parse::<usize>() {
                    let pem = fs::read_to_string(&path)?;
                    if let Ok(pub_key) = RsaPublicKey::from_public_key_pem(&pem) {
                        if id >= keys.len() {
                            keys.resize(id + 1, None);
                        }
                        keys[id] = Some(pub_key);
                    }
                }
            }
        }
    }

    Ok(keys)
}
/*============================================= PUBLISH ============================================= */


async fn publish_validated_bid(
    channel: &Channel, 
    bid: &Bid
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(bid)?;
    channel
        .basic_publish(
            "",
            "lance_validado",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?
        .await?;
    println!("Published Validated bid on lance_validado");
    dbg!(bid);

    Ok(())
}

async fn publish_winner_bid(
    channel: &Channel, 
    bid: &Bid
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(bid)?;
    channel
        .basic_publish(
            "",
            "leilao_vencedor",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?
        .await?;

    println!("Published Winner bid on leilao_vencedor");
    dbg!(bid);

    Ok(())
}  

/*============================================= PUBLISH - END ============================================= */

/*============================================= BID VERIFICATION ============================================= */

async fn is_bid_valid(
    bid: &Bid,
    auctions: &Arc<Mutex<Vec<Auction>>>,
    bids: &Arc<Mutex<Vec<Bid>>>,
    public_key: RsaPublicKey
) -> bool {
    let auctions = auctions.lock().await;
    let auction_opt = auctions.iter().find(|a| a.id == bid.auction_id && a.status);

    if auction_opt.is_none() {
        println!("Auction not found or inactive, bid invalid");
        return false;
    }

    let bids = bids.lock().await;
    let highest_bid_opt = bids
        .iter()
        .filter(|b| b.auction_id == bid.auction_id)
        .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

    if let Some(highest_bid) = highest_bid_opt {
        if bid.value <= highest_bid.value {
            println!("Bid value {} is not higher than current highest bid {}, bid invalid", bid.value, highest_bid.value);
            return false;
        }
    }

    print!("Verifying bid signature... {}", verify_bid(bid, public_key.clone()));
    verify_bid(bid, public_key)

}

fn verify_bid(bid: &Bid, public_key: RsaPublicKey) -> bool {
    let content = format!("{}:{}:{}", bid.auction_id, bid.client_id, bid.value).into_bytes();
    let hashed = Sha256::digest(content);

    let thing: String = hashed.iter().map(|b| format!("{:02x}", b)).collect();
    println!("verifying->{}:{}:{}\n{}", bid.auction_id, bid.client_id, bid.value, thing);

    let signature_bytes = match general_purpose::STANDARD.decode(&bid.signature) {
        Ok(sig) => sig,
        Err(_) => {
            println!("Failed to decode base64 signature");
            return false;
        }
    };

    public_key
        .verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &signature_bytes)
        .is_ok()
}

/*============================================= BID VERIFICATION - END ============================================= */
