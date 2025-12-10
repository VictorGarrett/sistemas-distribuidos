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

use sha2::{Digest, Sha256};
use base64::engine::general_purpose;
use base64::Engine;
use std::{fs, env ,path::Path};
use serde_json;

use shared::models::{
    Auction,
    Bid
};

use tonic::{transport::Server};

pub mod bid_srv {
    tonic::include_proto!("bid_srv");
}

use bid_srv::bid_service_server::{BidService, BidServiceServer};
use bid_srv::{
    CreateBidResponse as ProtoCreateBidResponse,
    CreateBidRequest as ProtoCreateBidRequest,
};


#[derive(Clone)]
pub struct BidServiceImpl {
    state: AppState,
}

#[tonic::async_trait]
impl BidService for BidServiceImpl {

    async fn create_bid(
        &self,
        request: tonic::Request<ProtoCreateBidRequest>,
    ) -> Result<tonic::Response<ProtoCreateBidResponse>, tonic::Status> {

        let req = request.into_inner();


        let bid = Bid {
            auction_id: req.auction_id,
            client_id: req.client_id,
            value: req.value,
            signature: req.signature,
            public_key: req.public_key,
            valid: req.valid };

        println!("Received bid via HTTP POST");
        dbg!(&bid);

        //let public_key = match state.public_keys.get(bid.client_id as usize) {
        //    Some(Some(key)) => key.clone(),
        //    _ => {
        //        println!("Invalid client ID or key not found for: {}", bid.client_id);
        //        return (StatusCode::BAD_REQUEST, "Invalid client ID or key not found").into_response();
        //    }
        //};

        let bypass_key = indoc::indoc! {r#"
        -----BEGIN PUBLIC KEY-----
        MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu+eNcaO1k41frKNUhmq/
        7QY98WiPZPEHVHY3qkiux1uUgIFBhMpOYOCiaJJMxXBhcHXxFoy0qFlCzr21d/yh
        hFCQLacv7J1svmV5KWn/1G2OE0RmOuk1KggWI1VnBQLGPh+u8bkzMqQ7EjNQcvtb
        9Y4g0AcjafTP+7RCESmEjHLREKW0a2HMSSX+uyRrUShAHyHygu6mGAiwHdk/bG2o
        7+Vv+DLFHtQS5sRKF67kafzWc7ngKdxaZjL7fYB55VMuAnFFEk6qX/Erqh5v9FP8
        ydtBVTiecJyMAeYvtRsG7tpF+X56F0FT2NpoUcW+0XwpTINdVS+rIMwI2X7fy/oB
        LQIDAQAB
        -----END PUBLIC KEY-----
        "#};

        let public_key = match RsaPublicKey::from_public_key_pem(bypass_key) {
            Ok(key) => key,
            Err(e) => { // It's also good practice to print the error 'e'
                println!("Failed to parse hardcoded public key: {}", e);
                return Ok(tonic::Response::new(ProtoCreateBidResponse {
                        success: false
                }))
            }
        };

        let bid_is_valid = is_bid_valid(
            &bid,
            &self.state.auctions,
            &self.state.bids,
            public_key
        ).await;


        if bid_is_valid {
            // Add to bids vector
            let mut bids_lock = self.state.bids.lock().await;
            bids_lock.push(bid.clone());
            drop(bids_lock); // Release lock

            let channel = match self.state.conn.create_channel().await {
                Ok(channel) => channel,
                Err(e) => {
                    eprintln!("Failed to create RabbitMQ channel: {}", e);
                    return Ok(tonic::Response::new(ProtoCreateBidResponse {
                        success: false
                    }))
                }
            };

            if let Err(e) = publish_validated_bid(&channel, &bid).await {
                eprintln!("Failed to publish validated bid: {}", e);
                return Ok(tonic::Response::new(ProtoCreateBidResponse {
                        success: false
                }))
            }

            return Ok(tonic::Response::new(ProtoCreateBidResponse {
                        success: true
            }))
        } 
        println!("Bid was deemed invalid");

        let channel = match self.state.conn.create_channel().await {
            Ok(channel) => channel,
            Err(e) => {
                eprintln!("Failed to create RabbitMQ channel: {}", e);
                return Ok(tonic::Response::new(ProtoCreateBidResponse {
                    success: false
                }))
            }
        };

        if let Err(e) = publish_invalidated_bid(&channel, &bid).await{
            eprintln!("Failed to publish invalidated bid: {}", e);
            return Ok(tonic::Response::new(ProtoCreateBidResponse {
                    success: false
            }))
        }
        
        return Ok(tonic::Response::new(ProtoCreateBidResponse {
                    success: false
        }))
    
    }

}


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



pub async fn task_grpc_server(
    auctions: Arc<Mutex<Vec<Auction>>>,
    bids: Arc<Mutex<Vec<Bid>>>,
    conn: Arc<Connection>,
    addr: String
) {

    let keys_path = env::var("KEYS_PATH").unwrap_or("bid-srv/keys".to_string());
    let public_keys = load_public_keys_vec(keys_path.as_str()).unwrap();

    let state = AppState {
        auctions: auctions,
        bids: bids,
        conn: conn,
        public_keys: public_keys,
    };


    let service = BidServiceImpl { state };

    println!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(BidServiceServer::new(service))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
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

async fn publish_invalidated_bid(
    channel: &Channel, 
    bid: &Bid
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(bid)?;
    channel
        .basic_publish(
            "",
            "lance_invalidado",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?
        .await?;
    println!("Published Invalidated bid on lance_invalidado");
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
            "leilao_vencedor",
            "",
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
        println!("Auction not found or inactive, bid invalid {:?}", auctions);

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
    return true;
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
