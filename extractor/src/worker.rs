use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use serde_json;

#[derive(Debug, serde::Deserialize)]
struct NewFileProcessQueue {
    file: String,
    start_page: u32,
    page_count: u32,
    piority: i32,
}


pub async fn run_worker() -> Result<()> {
    let connection_url = std::env::var("RABBITMQ_URL").unwrap();
    let connection_url = connection_url.as_str();
    // Connect to RabbitMQ server
    let mut connection = Connection::insecure_open(connection_url)?;
    println!("Connected to RabbitMQ");

    // Open a channel
    let channel = connection.open_channel(None)?;

    // Declare the queue (ensure it matches the name in your Node.js app)
    let queue = channel.queue_declare("NEW_FILE_EXTRACT", QueueDeclareOptions {
            durable: true, // This ensures the queue matches the Node.js settings
            ..QueueDeclareOptions::default()
        })?;

    // Start consuming messages
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages...");

    // Process messages as they arrive
    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                // Parse the JSON message into the struct
                let msg: NewFileProcessQueue = serde_json::from_slice(&delivery.body).unwrap();
                println!("Received: {:?}", msg);

                // Process the file (placeholder for your logic)
                println!("Processing file: {}", msg.file);

                // Acknowledge the message
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }
    println!("closing connection");
    connection.close()
}
