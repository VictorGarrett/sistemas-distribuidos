
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