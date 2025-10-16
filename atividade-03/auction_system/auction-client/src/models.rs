use rsa::{ RsaPrivateKey};


#[derive(Clone)]
pub struct Client{
    pub id: u32,
    pub subscribed_auctions: Vec<u32>,
    pub private_key: RsaPrivateKey,
    pub public_key: String,
    pub notification_queue_name: String,
}

#[derive(Clone, Copy)]
pub enum CliCommand{
    Subscribe{
        auction_id: u32
    },
    MakeBid{
        auction_id: u32,
        value: f64
    },
}

impl CliCommand{
    pub fn get_auction_id(&self) -> u32{
        match self{
            Self::Subscribe { auction_id } => *auction_id,
            Self::MakeBid { auction_id, .. } => *auction_id
        }
    }
}

pub enum Destructured {
    MakeBid(u32, f64),
    Subscribe(u32),
}

impl CliCommand {
    pub fn destructure(self) -> Option<Destructured> {
        match self {
            Self::MakeBid { auction_id, value } => Some(Destructured::MakeBid(auction_id, value)),
            Self::Subscribe { auction_id } => Some(Destructured::Subscribe(auction_id)),
        }
    }
}

