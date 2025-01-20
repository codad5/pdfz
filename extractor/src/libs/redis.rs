use redis::{AsyncCommands, Client, RedisResult};
use std::env;

// Initialize Redis client
pub async fn get_redis_client() -> RedisResult<Client> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    Client::open(redis_url)
}

// Check if a file is in process
pub async fn is_file_in_process(client: &Client, file_id: &str) -> RedisResult<bool> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    let exists: Option<String> = con.get(&key).await?;

    if let Some(progress) = exists {
        if progress.parse::<i32>().unwrap_or(0) >= 100 {
            // If progress is 100, it is done processing, so delete the key and return false
            let _: () = con.del(&key).await?;
            return Ok(false);
        }
        return Ok(true);
    }
    Ok(false)
}

// Check if the process is done
pub async fn is_process_done(client: &Client, file_id: &str) -> RedisResult<bool> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    let exists: Option<String> = con.get(&key).await?;
    Ok(exists.map_or(false, |progress| progress.parse::<i32>().unwrap_or(0) >= 100))
}

// Set the progress for a file
pub async fn set_progress(client: &Client, file_id: &str, progress: i32) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    con.set(key, progress).await
}

// Get the progress for a file
pub async fn get_progress(client: &Client, file_id: &str) -> RedisResult<i32> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    let progress: Option<String> = con.get(&key).await?;
    Ok(progress.map_or(0, |p| p.parse::<i32>().unwrap_or(0)))
}

// Mark a file as processing with a TTL (default: 3600 seconds)
pub async fn mark_file_as_processing(client: &Client, file_id: &str, ttl: u64) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    con.set_ex(key, 0, ttl).await
}

// mark as done
pub async fn mark_as_done(client: &Client, file_id: &str) -> RedisResult<()> {
   set_progress(client, file_id, 100).await
}