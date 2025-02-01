use std::sync::Arc;

use amiquip::{Connection, ConsumerMessage, ConsumerOptions, Delivery, QueueDeclareOptions, Result as AmiqpResult};
use lapin::message;
use redis::Client;
use tokio::sync::Semaphore;

use crate::{libs::redis::get_redis_client, types::engine_handler::Engines, worker::NewFileProcessQueue};

pub struct RabbitMQFileProcessor {
    rabbit_mq_conn : Connection,
}  


impl RabbitMQFileProcessor {
    pub fn new(rabbit_mq_conn: Connection) -> Self {
        return Self{
            rabbit_mq_conn, 
        };
    }

    pub async fn listen_for_messages(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let redis_client = Arc::new(get_redis_client().await.unwrap());
        println!("Connected to RabbitMQ");
        
        // Open a channel
        let channel = self.rabbit_mq_conn.open_channel(None)?;
        
        // Declare the queue
        //TODO: make the new key name variable in .env
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
                    let message: Result<NewFileProcessQueue, &str> = Self::get_message(&delivery);
                    if message.is_ok() {
                        let message = message.unwrap();
                        let engine = Engines::from(message.engine.as_str()).unwrap();
                        engine.handle(message, &redis_client, &semaphore).await;
                    }
                    consumer.ack(delivery)?;
                }, 
                other => {
                    println!("Cznsumer ended: {:?}", other);
                    break;
                }
            }
        }

        self.close_conn();
        Ok(())
    
    }

    fn get_message(delivery : &Delivery) -> Result<NewFileProcessQueue, &'static str> {
        let msg = serde_json::from_slice::<NewFileProcessQueue>(&delivery.body)
            .map_err(|_| "Failed to parse message")?;
        Ok(msg)
    }

    fn close_conn(self) -> AmiqpResult<()> {
        self.rabbit_mq_conn.close()
    }
}