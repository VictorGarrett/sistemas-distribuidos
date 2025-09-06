use serde::{Serialize, Deserialize};
use rsa::{ RsaPrivateKey};


#[derive(Serialize, Deserialize, Clone)]
pub struct Bid {
    pub auction_id: u32,
    pub client_id: u32,
    pub value: f64,
    pub signature: String,
    pub public_key: String,
    pub valid: bool
}



#[derive(Serialize, Deserialize)]
pub struct Auction {
    pub id: u32,
    pub item: String,
    pub start_timestamp: u128,
    pub end_timestamp: u128,
    pub status: bool
}

impl Auction {
    pub fn new(
        id: u32, 
        item: String,
        start_timestamp: u128,
        end_timestamp: u128
    ) -> Self {
        Auction {
            id,
            item,
            start_timestamp,
            end_timestamp,
            status: true
        }
    }
}

pub struct Client{
    pub id: u32,
    pub subscribed_auctions: Vec<u32>,
    pub private_key: RsaPrivateKey,
    pub public_key: String,
    pub notification_queue_name: String,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Notification{
    notification_type: NotificationType,
    data: NotificationData
}

impl Notification{
    pub fn from_bid(bid: &Bid, notification_type: NotificationType) -> Notification{
        Notification { 
            notification_type, 
            data: NotificationData { 
                auction_id: bid.auction_id, 
                client_id: bid.client_id, 
                bid_value: bid.value 
            } 
        }
    }

    pub fn get_auction_id(&self) -> u32{
        return self.data.auction_id;
    }
    pub fn get_client_id(&self) -> u32{
        return self.data.client_id;
    }
    pub fn get_bid_value(&self) -> f64{
        return self.data.bid_value;
    }

    pub fn get_notification_type(&self) ->NotificationType{
        return self.notification_type.clone();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum NotificationType{
    NewBid,
    AuctionWinner
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationData{
    auction_id: u32,
    client_id: u32,
    bid_value: f64,
}

pub enum CliCommand{
    Subscribe{
        auction_id: String
    },
    MakeBid{
        auction_id: String,
        value: f64
    },
}

pub enum Destructured {
    MakeBid(String, f64),
    Subscribe(String),
}

impl CliCommand {
    pub fn destructure(self) -> Option<Destructured> {
        match self {
            Self::MakeBid { auction_id, value } => Some(Destructured::MakeBid(auction_id, value)),
            Self::Subscribe { auction_id } => Some(Destructured::Subscribe(auction_id)),
        }
    }
}
