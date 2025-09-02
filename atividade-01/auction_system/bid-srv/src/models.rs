use std::time::{SystemTime, UNIX_EPOCH};

pub struct Bid {
    pub aution_id: u32,
    pub client_id: u32,
    pub value: f64,
    pub signature: String,
    pub valid: bool
}

pub struct Auction{
    pub id: u32,
    pub item: String,
    pub created_timestamp: u128,
    pub end_timestamp: Option<u128>,
    pub is_active: bool,
}

impl Auction{
    pub fn new(id: u32, item: String) -> Self{
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Auction{
            id,
            item,
            created_timestamp: now,
            end_timestamp: None,
            is_active: true,
        }
    }
    
    pub fn set_inactive(&mut self){
        self.is_active = false;
        self.end_timestamp = Some(
            SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
        );
    }
}