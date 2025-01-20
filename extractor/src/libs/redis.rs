use redis::{AsyncCommands, Client, RedisResult};
use std::env;

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Pending,
    Done,
    Failed,
}

impl Status {
    fn to_string(&self) -> String {
        match self {
            Status::Pending => "pending".to_string(),
            Status::Done => "done".to_string(),
            Status::Failed => "failed".to_string(),
        }
    }

    fn from_string(s: &str) -> Self {
        match s {
            "done" => Status::Done,
            "failed" => Status::Failed,
            _ => Status::Pending,
        }
    }
}

// impl display for Status
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Initialize Redis client
pub async fn get_redis_client() -> RedisResult<Client> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    Client::open(redis_url)
}

// Check if a file is in process
pub async fn is_file_in_process(client: &Client, file_id: &str) -> RedisResult<bool> {
    let status = get_file_status(client, file_id).await?;
    Ok(status == Status::Pending)
}

// Check if the process is done
pub async fn is_process_done(client: &Client, file_id: &str) -> RedisResult<bool> {
    let status = get_file_status(client, file_id).await?;
    Ok(status == Status::Done)
}

// Set the status for a file
async fn set_status(client: &Client, file_id: &str, status: Status) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    println!("Redis ==> {} ==> status {}", file_id, status);
    con.set(key, status.to_string()).await
}

// Get the status for a file
pub async fn get_file_status(client: &Client, file_id: &str) -> RedisResult<Status> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    let status: Option<String> = con.get(&key).await?;
    Ok(status.map_or(Status::Pending, |s| Status::from_string(&s)))
}

// Start processing a file with a TTL (default: 3600 seconds)
pub async fn start_file_process(client: &Client, file_id: &str, ttl: u64) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    con.set_ex(key, Status::Pending.to_string(), ttl).await
}

// Mark a file as done
pub async fn mark_as_done(client: &Client, file_id: &str) -> RedisResult<()> {
    set_status(client, file_id, Status::Done).await
}

// Mark a file as failed
pub async fn mark_as_failed(client: &Client, file_id: &str) -> RedisResult<()> {
    set_status(client, file_id, Status::Failed).await
}
pub async fn mark_as(client: &Client, file_id: &str, status: Status) -> RedisResult<()> {
    set_status(client, file_id, status).await
}