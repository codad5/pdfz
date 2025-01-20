use std::sync::Arc;

use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use serde_json;
use tokio::sync::Semaphore;
use tokio::task;
use crate::libs::extractor::extract_file;
use crate::libs::redis::{get_redis_client, mark_as_done};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewFileProcessQueue {
    pub file: String,
    pub start_page: u32,
    pub page_count: u32,
}

pub async fn run_worker() -> Result<()> {
    let connection_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");

    // Connect to RabbitMQ server
    let mut connection = Connection::insecure_open(&connection_url)?;
    let redis_client = get_redis_client().await.unwrap();
    println!("Connected to RabbitMQ");

    // Open a channel
    let channel = connection.open_channel(None)?;

    // Declare the queue
    let queue = channel.queue_declare("NEW_FILE_EXTRACT", QueueDeclareOptions {
        durable: true,
        ..QueueDeclareOptions::default()
    })?;

    // Start consuming messages
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages...");

    // Semaphore to limit concurrency to 5 tasks
    // let semaphore = Semaphore::new(5);
    let semaphore = Arc::new(Semaphore::new(5));

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                // Parse the JSON message into the struct
                let msg: NewFileProcessQueue = serde_json::from_slice(&delivery.body).unwrap();
                println!("Received: {:?}", msg);

                // Acquire a permit to ensure we don't exceed concurrency limit
                let redis_client = redis_client.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                // Spawn a new async task to process the file
                task::spawn(async move {
                    println!("Processing file: {}", msg.file);

                    // Process the file (your logic goes here)
                    extract_file(msg.clone()).await;
                    // id is the file name without extension
                    let id = msg.file.split('.').collect::<Vec<&str>>()[0];
                    mark_as_done(&redis_client, id).await.unwrap();

                    // Permit is dropped here automatically, allowing another task to start
                    drop(permit);
                });

                // Acknowledge the message
                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    println!("Closing connection");
    connection.close()
}
