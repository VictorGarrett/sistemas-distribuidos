use serde::{Serialize, Deserialize};


/* ========================================= AUCITON ========================================= */

#[derive(Clone, Serialize, Deserialize, Debug)]
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

/* ========================================= BID ========================================= */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bid {
    pub auction_id: u32,
    pub client_id: u32,
    pub value: f64,
    pub signature: String,
    pub public_key: String,
    pub valid: bool
}


/* ========================================= NOTIFICATION ========================================= */

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NotificationType{
    NewBid,
    AuctionWinner
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationData{
    auction_id: u32,
    client_id: u32,
    bid_value: f64,
}