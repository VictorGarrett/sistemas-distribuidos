use lapin::{
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use futures_lite::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to RabbitMQ
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Connected to RabbitMQ!");

    // 2. Create a channel
    let channel = conn.create_channel().await?;

    // 3. Declare a queue
    let queue = channel
        .queue_declare(
            "test_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue.name());

    // 4. Publish a message
    let payload = b"Hello from Rust & Lapin!";
    channel
        .basic_publish(
            "",
            "test_queue",
            BasicPublishOptions::default(),
            payload, // <-- changed here
            BasicProperties::default(),
        )
        .await?
        .await?; // Wait for confirmation
    println!("Message published!");

    // 5. Consume the message
    let mut consumer = channel
        .basic_consume(
            "test_queue",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("Waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        println!("Received: {:?}", std::str::from_utf8(&delivery.data)?);
        delivery.ack(Default::default()).await?;
        break; // Exit after first message
    }

    Ok(())
}
