use std::sync::Arc;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result as AmiqpResult};
use serde_json;
use tokio::sync::Semaphore;
use tokio::task;
use anyhow::{Result, Error};
use crate::libs::extractor::extract_file;
use crate::libs::redis::{get_redis_client, mark_as, mark_as_done, mark_as_failed, Status};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewFileProcessQueue {
    pub file: String,
    pub start_page: u32,
    pub page_count: u32,
}

// Helper function to process a single message
async fn process_message(
    msg: NewFileProcessQueue,
    redis_client: Arc<redis::Client>,
) -> Result<(), Error> {
    let mut status = Status::Done;
    if let Err(e) = extract_file(msg.clone()).await {
        status = Status::Failed;
    }

    let id = msg.file.split('.').next().unwrap_or("");
    if let Err(e) = mark_as(&redis_client, id, status.clone()).await {
        return Err(Error::msg(format!("Error marking as {}:  {}", status, e)));
    }

    Ok(())
}

pub async fn run_worker() -> AmiqpResult<()> {
    let connection_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");
    
    // Connect to RabbitMQ server
    let mut connection = Connection::insecure_open(&connection_url)?;
    let redis_client = Arc::new(get_redis_client().await.unwrap());
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
    
    let semaphore = Arc::new(Semaphore::new(5));
    
    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                // Parse the JSON message into the struct
                let msg: NewFileProcessQueue = match serde_json::from_slice(&delivery.body) {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Error parsing message: {}", e);
                        consumer.ack(delivery)?;
                        continue;
                    }
                };
                println!("Received: {:?}", msg);
                
                let redis_client = redis_client.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                
                // Spawn task with explicit Send error handling
                task::spawn(async move {
                    println!("Processing file: {}", msg.file);
                    
                    match process_message(msg.clone(), redis_client).await {
                        Ok(_) => println!("File processed successfully"),
                        Err(e) => println!("{}", e),
                    }
                    
                    drop(permit);
                });
                
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