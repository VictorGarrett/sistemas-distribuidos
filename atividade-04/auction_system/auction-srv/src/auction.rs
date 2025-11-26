use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
pub struct Auction{
    pub id: u32,
    pub item: String,
    pub start_timestamp: u128,
    pub end_timestamp: Option<u128>,
    pub status: bool,
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
            start_timestamp: now,
            end_timestamp: None,
            status: true,
        }
    }
    
    pub fn set_inactive(&mut self){
        self.status = false;
        self.end_timestamp = Some(
            SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
        );
    }
}