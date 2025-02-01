use amiquip::Connection;
use crate::controllers::queue_handler::RabbitMQFileProcessor;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct NewFileProcessQueue {
    pub file: String,
    pub start_page: u32,
    pub page_count: u32,
    pub engine: String,
}




pub async fn run_worker() -> Result<(), amiquip::Error> {
    let connection_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");
    
//     // Connect to RabbitMQ server
    let connection: Connection = Connection::insecure_open(&connection_url)?;
    let mqservice = RabbitMQFileProcessor::new(connection);
    if let Err(e) = mqservice.listen_for_messages().await {
        println!("Error listening for messages: {}", e);
    }
    Ok(())
}

// pub async fn run_worker() -> AmiqpResult<()> {
//     let connection_url = std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL must be set");
    
//     // Connect to RabbitMQ server
//     let mut connection: Connection = Connection::insecure_open(&connection_url)?;
//     let redis_client = Arc::new(get_redis_client().await.unwrap());
//     println!("Connected to RabbitMQ");
    
//     // Open a channel
//     let channel = connection.open_channel(None)?;
    
//     // Declare the queue
//     //TODO: make the new key name variable in .env
//     let queue = channel.queue_declare("NEW_FILE_EXTRACT", QueueDeclareOptions {
//         durable: true,
//         ..QueueDeclareOptions::default()
//     })?;
    
//     // Start consuming messages
//     let consumer = queue.consume(ConsumerOptions::default())?;
//     println!("Waiting for messages...");
    
//     let semaphore: Arc<Semaphore> = Arc::new(Semaphore::new(5));
    
//     for message in consumer.receiver().iter() {
//         match message {
//             ConsumerMessage::Delivery(delivery) => {
//                 // Parse the JSON message into the struct
//                 let msg: NewFileProcessQueue = match serde_json::from_slice(&delivery.body) {
//                     Ok(msg) => msg,
//                     Err(e) => {
//                         println!("Error parsing message: {}", e);
//                         consumer.ack(delivery)?;
//                         continue;
//                     }
//                 };
//                 println!("Received: {:?}", msg);
                
//                 let redis_client = redis_client.clone();
//                 let permit = semaphore.clone().acquire_owned().await.unwrap();
                
//                 // Spawn task with explicit Send error handling
//                 task::spawn(async move {
//                     println!("Processing file: {}", msg.file);
                    
//                     match process_message(msg.clone(), redis_client).await {
//                         Ok(_) => println!("File processed successfully"),
//                         Err(e) => println!("{}", e),
//                     }
                    
//                     drop(permit);
//                 });
                
//                 consumer.ack(delivery)?;
//             }
//             other => {
//                 println!("Consumer ended: {:?}", other);
//                 break;
//             }
//         }
//     }
    
//     println!("Closing connection");
//     connection.close()
// }


// // Helper function to process a single message
// async fn process_message(
//     msg: NewFileProcessQueue,
//     redis_client: Arc<redis::Client>,
// ) -> Result<(), Error> {
//     let mut status = Status::Done;
//     if let Err(e) = extract_file(msg.clone()).await {
//         println!("Error processing file: {}", e);
//         status = Status::Failed;
//     }

//     let id = msg.file.split('.').next().unwrap_or("");
//     if let Err(e) = mark_as(&redis_client, id, status.clone()).await {
//         return Err(Error::msg(format!("Error marking as {}:  {}", status, e)));
//     }
//     return match status {
//         Status::Done => Ok(()),
//         Status::Failed => Err(Error::msg("Failed to process file")),
//         _ => Err(Error::msg("Unknown status")),
        
//     }
// }
