use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};

use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use rsa::Pkcs1v15Sign;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    let pem = "-----BEGIN RSA PRIVATE KEY-----
MIICWgIBAAKBgGmcA0BGDhveEY6+dmEo1lil2NpB0Y8NpXdpBUi5DZLbk9Sg/sTv
8z/AURr99a7MKAVFFngHTioOwxB5ruwhdvuFKKyqTnzYK2dv87WJ7GqqUda2rlhB
my4CCOXSS+YLqgdQYj4QesBDOC9ojdFaIPGIyp77J4iHAoICxN+y+Rn9AgMBAAEC
gYAarXFYzBmGSptuzogC1RkIPaTAxX2VQGI6/sl57F0Uauk1/hE9WEu/H+qdAegM
5r95TVF2som5MA9wWvyn43A1l+zjuHYZIZZTNguhbDZ+oFEWFxERzzqe1EF+DQ25
n4vV9H2Iww2KdbyC8RuabK/QRqeTBH+JNOw5Ng84Hkbg9QJBAKxSpaPsHeHR+r/Y
vqiuTqnzxp6WKkoJ5t+w6ZmCLisoAUpyjZcMKJfdditf1ypCL5CpbMxV/Dh0Wt8E
bvo2lwsCQQCc5EJqTCP9LVs9Pn4UoSQLoR9K39zu4sE0rbXyXsLx2dlb992ednCm
f89vZZ9hVaR5EUn+51s34QUxUnXdCZgXAkBeoniy4B29AUr6hraV/jvXG7hNKVyK
EowG9qojEpn2O18SGnzlodi9JfMaeOS6IWTrxg+o2+PKwSOSbGXh5Y7nAkA+ywTh
8nN9A0g/LOHdc9kvZl9V4l9UpSDa6qOly9OOZLigHIZww8q2ePUXCr9Nf6+CXS8W
fJZ/uOoRIYXW394lAkA89SZ9iU/6GkTG6z5hzENPfiuiPGV3Ytp+TgbDxprVhGrv
sdpoIwdgJDlOHWpRethjNiNx7FITofCcsqC/8Mp7
-----END RSA PRIVATE KEY-----";

let private_key = RsaPrivateKey::from_pkcs1_pem(pem)?;


    // Connect to RabbitMQ
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel.queue_declare("test_queue", QueueDeclareOptions::default(), FieldTable::default()).await?;

    let message = b"Hello from Rust & Lapin with pure RSA!";

    let hashed = Sha256::digest(message);

    // Sign
    let signature = private_key.sign(
        Pkcs1v15Sign::new_unprefixed(),
        &hashed,
    )?;

    let payload = json!({
        "message": String::from_utf8_lossy(message),
        "signature": general_purpose::STANDARD.encode(signature),
    })
    .to_string();

    channel.basic_publish(
        "",
        "test_queue",
        BasicPublishOptions::default(),
        payload.as_bytes(),
        BasicProperties::default()
    ).await?.await?;

    println!("Signed message sent!");
    Ok(())
}