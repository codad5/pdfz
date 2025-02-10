use std::sync::Arc;
use amiquip::{Channel, Connection, ConsumerMessage, ConsumerOptions, Delivery, QueueDeclareOptions, Result as AmiqpResult};
use redis::Client;
use tokio::sync::Semaphore;

use crate::{libs::redis::get_redis_client, types::engine_handler::Engines, worker::NewFileProcessQueue};

pub struct RabbitMQFileProcessor {
    rabbit_mq_conn: Connection,
}

impl RabbitMQFileProcessor {
    pub fn new(rabbit_mq_conn: Connection) -> Self {
        Self {
            rabbit_mq_conn,
        }
    }

    pub async fn listen_for_messages(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let redis_client = Arc::new(get_redis_client().await.unwrap());
        println!("Connected to RabbitMQ");

        // Open a channel for the main process
        let channel = self.rabbit_mq_conn.open_channel(None)?;

        // Spawn the Ollama model pull listener with a new channel
        self.listen_for_ollama_model_pull();

        // Declare the queue
        let new_file_extract_queue = channel.queue_declare(
            "NEW_FILE_EXTRACT",
            QueueDeclareOptions {
                durable: true,
                ..QueueDeclareOptions::default()
            },
        )?;

        // Start consuming messages
        let file_extract_queue_consumer = new_file_extract_queue.consume(ConsumerOptions::default())?;
        println!("Waiting for messages...");

        let semaphore = Arc::new(Semaphore::new(5));

        for message in file_extract_queue_consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let message: Result<NewFileProcessQueue, &str> = Self::get_message(&delivery);
                    if message.is_ok() {
                        let message = message.unwrap();
                        let engine = Engines::from(message.engine.as_str()).unwrap();
                        engine.handle(message, &redis_client, &semaphore).await;
                    }
                    file_extract_queue_consumer.ack(delivery)?;
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        self.close_conn()?;
        Ok(())
    }

    fn get_message(delivery: &Delivery) -> Result<NewFileProcessQueue, &'static str> {
        let msg = serde_json::from_slice::<NewFileProcessQueue>(&delivery.body)
            .map_err(|_| "Failed to parse message")?;
        Ok(msg)
    }

    fn close_conn(self) -> AmiqpResult<()> {
        self.rabbit_mq_conn.close()
    }

    fn listen_for_ollama_model_pull(&mut self) {
        // Create a new channel specifically for this task
        let channel = self.rabbit_mq_conn.open_channel(None);
        if let Ok(channel) = channel {
            tokio::spawn(async move {
                if let Ok(queue) = channel.queue_declare(
                    "OLLAMA_MODEL_PULL",
                    QueueDeclareOptions {
                        durable: true,
                        ..QueueDeclareOptions::default()
                    },
                ) {
                    if let Ok(consumer) = queue.consume(ConsumerOptions::default()) {
                        // Handle consumer messages here
                        println!("Ollama model pull queue consumer started");
                        
                        // You might want to add a loop here to handle messages
                        for message in consumer.receiver().iter() {
                            match message {
                                ConsumerMessage::Delivery(delivery) => {
                                    // Handle the delivery
                                    if let Err(e) = consumer.ack(delivery) {
                                        println!("Failed to acknowledge message: {:?}", e);
                                    }
                                }
                                other => {
                                    println!("Ollama consumer ended: {:?}", other);
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}