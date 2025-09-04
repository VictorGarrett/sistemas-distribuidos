use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Bid {
    pub auction_id: u32,
    pub client_id: u32,
    pub value: f64,
    pub signature: String,
    pub public_key: String,
    pub valid: bool
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