use std::{sync::Arc};
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, Delivery, QueueDeclareOptions, Result as AmiqpResult};
use futures_lite::StreamExt;
use ollama_rs::{models::pull::PullModelStatusStream, Ollama};
use tokio::sync::Semaphore;

use crate::{libs::redis::{get_redis_client, mark_model_as_completed, mark_model_as_failed, update_model_progress}, types::engine_handler::Engines, worker::{NewFileProcessQueue, OllamaModelPull}};

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


    fn get_model_message(delivery: &Delivery) -> Result<OllamaModelPull, &'static str> {
        let msg = serde_json::from_slice::<OllamaModelPull>(&delivery.body)
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
            tokio::spawn(async move  {
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
                                    let message_detail = Self::get_model_message(&delivery);
                                    if message_detail.is_ok() {
                                        let message_detail = message_detail.unwrap();
                                        // TODO: integrate the model pull
                                        // download_model_stream(message_detail.name).await;
                                    }                                    
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

async fn download_model_stream(model_name : String){
// rt.block_on(async {
    let redis_client = get_redis_client().await;
    if  redis_client.is_err() {
        return;
    }
    let redis_client = redis_client.unwrap();
    let ollama = Ollama::default();
    let  model_stream = ollama.pull_model_stream(model_name.clone(), false).await;

        let mut model_stream : PullModelStatusStream = model_stream.unwrap();
        while let Some(d) = model_stream.next().await {
            if d.is_err() {
                println!("failed to download model : {}", model_name);
                let _ = mark_model_as_failed(&redis_client, &model_name).await;
                break;
            }

            let d = d.unwrap();
            let completed = match d.completed {
                Some(g) => g, 
                None => 1,
            };

            let total: u64 = match d.total {
                Some(g) => g, 
                None => 2,
            };
            update_model_progress(&redis_client, &model_name, completed, total).await;         
        }
        mark_model_as_completed(&redis_client, &model_name).await;
}