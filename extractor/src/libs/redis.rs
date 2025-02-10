use redis::{AsyncCommands, Client, RedisResult};
use tonic::client;
use std::env;

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Pending,
    Done,
    Failed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ModelStatus {
    Queued,
    Downloading,
    Completed,
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

impl ModelStatus {
    fn to_string(&self) -> String {
        match self {
            ModelStatus::Queued => "queued".to_string(),
            ModelStatus::Downloading => "downloading".to_string(),
            ModelStatus::Completed => "completed".to_string(),
            ModelStatus::Failed => "failed".to_string(),
        }
    }

    fn from_string(s: &str) -> Self {
        match s {
            "queued" => ModelStatus::Queued,
            "downloading" => ModelStatus::Downloading,
            "completed" => ModelStatus::Completed,
            "failed" => ModelStatus::Failed,
            _ => ModelStatus::Queued,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Initialize Redis client
pub async fn get_redis_client() -> RedisResult<Client> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    Client::open(redis_url)
}

// File Processing Functions
pub async fn is_file_in_process(client: &Client, file_id: &str) -> RedisResult<bool> {
    let status = get_file_status(client, file_id).await?;
    Ok(status == Status::Pending)
}

pub async fn is_process_done(client: &Client, file_id: &str) -> RedisResult<bool> {
    let status = get_file_status(client, file_id).await?;
    Ok(status == Status::Done)
}

async fn set_status(client: &Client, prefix: &str, id: &str, status: impl ToString) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("{}:{}", prefix, id);
    println!("Redis ==> {} ==> status {}", id, status.to_string());
    con.set(key, status.to_string()).await
}

pub async fn get_file_status(client: &Client, file_id: &str) -> RedisResult<Status> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    let status: Option<String> = con.get(&key).await?;
    Ok(status.map_or(Status::Pending, |s| Status::from_string(&s)))
}

pub async fn start_file_process(client: &Client, file_id: &str, ttl: u64) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("processing:{}", file_id);
    con.set_ex(key, Status::Pending.to_string(), ttl).await
}

pub async fn mark_as_done(client: &Client, file_id: &str) -> RedisResult<()> {
    set_status(client, "processing", file_id, Status::Done).await
}

pub async fn mark_as_failed(client: &Client, file_id: &str) -> RedisResult<()> {
    set_status(client, "processing", file_id, Status::Failed).await
}

// Model Download Functions
pub async fn is_model_downloading(client: &Client, model_name: &str) -> RedisResult<bool> {
    let status = get_model_status(client, model_name).await?;
    Ok(status == ModelStatus::Downloading)
}

pub async fn is_model_download_complete(client: &Client, model_name: &str) -> RedisResult<bool> {
    let status = get_model_status(client, model_name).await?;
    Ok(status == ModelStatus::Completed)
}

pub async fn get_model_status(client: &Client, model_name: &str) -> RedisResult<ModelStatus> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("model:status:{}", model_name);
    let status: Option<String> = con.get(&key).await?;
    Ok(status.map_or(ModelStatus::Queued, |s| ModelStatus::from_string(&s)))
}

pub async fn start_model_download(client: &Client, model_name: &str, ttl: u64) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let status_key = format!("model:status:{}", model_name);
    let progress_key = format!("model:progress:{}", model_name);
    
    con.set_ex(status_key, ModelStatus::Queued.to_string(), ttl).await?;
    con.set(progress_key, 0).await
}

pub async fn mark_as_downloading(client: &Client, model_name: &str) -> RedisResult<()> {
    set_status(client, "model:status", model_name, ModelStatus::Downloading).await
}

pub async fn mark_model_as_completed(client: &Client, model_name: &str) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    set_status(client, "model:status", model_name, ModelStatus::Completed).await?;
    let progress_key = format!("model:progress:{}", model_name);
    con.set(progress_key, 100).await
}

pub async fn mark_model_as_failed(client: &Client, model_name: &str) -> RedisResult<()> {
    set_status(client, "model:status", model_name, ModelStatus::Failed).await
}

pub async fn update_model_progress(client: &Client, model_name: &str, downloaded_bytes: u64, total_bytes: u64) -> RedisResult<()> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let progress_key = format!("model:progress:{}", model_name);
    
    let progress = if total_bytes == 0 {
        0
    } else {
        ((downloaded_bytes * 100) / total_bytes) as u32
    };
    
    con.set(&progress_key, progress).await?;
    
    if progress == 100 {
        mark_model_as_completed(client, model_name).await?;
    }
    
    Ok(())
}

pub async fn get_model_progress(client: &Client, model_name: &str) -> RedisResult<u32> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("model:progress:{}", model_name);
    let progress: Option<u32> = con.get(&key).await?;
    Ok(progress.unwrap_or(0))
}

pub async fn get_downloading_models(client: &Client) -> RedisResult<Vec<String>> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let pattern = "model:status:*";
    let keys: Vec<String> = con.keys(pattern).await?;
    
    let mut downloading_models = Vec::new();
    for key in keys {
        let model_name = key.replace("model:status:", "");
        let status = get_model_status(client, &model_name).await?;
        if status == ModelStatus::Downloading {
            downloading_models.push(model_name);
        }
    }
    
    Ok(downloading_models)
}

// Progress tracking for file processing
pub async fn mark_progress(file_id: &str, page: u32, total: u32) -> RedisResult<()> {
    let client = get_redis_client().await.unwrap();
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("progress:{}", file_id);
    let value = if page == 0 || total == 0 {
        0
    } else {
        (page * 100) / total
    };
    
    if value == 100 {
        mark_as_done(&client, file_id).await?;
    }
    
    con.set(key, value).await
}

pub async fn get_progress(client: &Client, prefix: &str, id: &str) -> RedisResult<u32> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let key = format!("{}:progress:{}", prefix, id);
    let progress: Option<u32> = con.get(&key).await?;
    Ok(progress.unwrap_or(0))
}