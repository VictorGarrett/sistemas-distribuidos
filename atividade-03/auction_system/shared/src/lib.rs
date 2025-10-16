

pub mod models;

#[cfg(test)]
mod tests {
    use crate::models::Auction;
    #[test]
    fn test_auction_creation() {
        let auction = Auction {
            id: 1,
            item: "Test".to_string(),
            start_timestamp: 0,
            end_timestamp: 0,
            status: false
        };
        assert_eq!(auction.id, 1);
    }
}

