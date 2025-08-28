use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://guest:guest@127.0.0.1:5672/%2f";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    println!("Publisher connected to RabbitMQ!");

    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "test_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    loop {
        let payload = b"Hello from the Publisher!";
        channel
            .basic_publish(
                "",
                "test_queue",
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?
            .await?;

        println!("Message sent!");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}