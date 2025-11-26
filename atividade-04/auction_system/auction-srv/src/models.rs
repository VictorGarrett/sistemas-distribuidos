use shared::models::Auction;


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

    pub fn to_auction(self, id: u32) -> Result<Auction, String>{
        if let Some((item, start_timestamp, end_timestamp)) = self.destructure(){
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