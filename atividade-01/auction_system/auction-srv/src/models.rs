use serde::{Serialize, Deserialize};

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

    pub fn from_cmd(cmd: CliCommand, id: u32) -> Result<Auction, String>{
        if let Some((item, start_timestamp, end_timestamp)) = cmd.destructure(){
            return Ok(Auction{
                id,
                item,
                start_timestamp: start_timestamp as u128,
                end_timestamp: end_timestamp as u128,
                status: true
            })
        }
        Err("Cannot create auction from command".to_string())
    }
}

pub enum CliCommand{
    CreateAuction{
        item: String,
        start_timestamp: u64,
        end_timestamp: u64
    },
    ListAuctions,
}

impl CliCommand{
    pub fn destructure(self) -> Option<(String, u64, u64)>{
        match self {
            Self::ListAuctions => None,
            Self::CreateAuction { item, start_timestamp, end_timestamp } => Some((item, start_timestamp, end_timestamp))
        }
    }
}