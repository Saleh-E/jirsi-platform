//! Background Job Queue System
//! 
//! Redis-based job queue for async task processing

use redis::{Client, AsyncCommands};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub max_retries: u32,
    pub retry_count: u32,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

pub struct JobQueue {
    client: Client,
    queue_name: String,
}

impl JobQueue {
    pub async fn new(redis_url: &str, queue_name: &str) -> Result<Self, JobError> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            queue_name: queue_name.to_string(),
        })
    }
    
    /// Enqueue a new job
    pub async fn enqueue(
        &self,
        job_type: String,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<Uuid, JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let job = Job {
            id: Uuid::new_v4(),
            job_type,
            payload,
            max_retries,
            retry_count: 0,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            error: None,
        };
        
        let job_json = serde_json::to_string(&job)?;
        
        // Add to queue
        conn.rpush(&self.queue_name, &job_json).await?;
        
        // Store job details
        conn.set_ex(
            format!("job:{}", job.id),
            &job_json,
            86400, // 24 hours
        ).await?;
        
        Ok(job.id)
    }
    
    /// Schedule a job for future execution
    pub async fn schedule(
        &self,
        job_type: String,
        payload: serde_json::Value,
        scheduled_at: DateTime<Utc>,
        max_retries: u32,
    ) -> Result<Uuid, JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let job = Job {
            id: Uuid::new_v4(),
            job_type,
            payload,
            max_retries,
            retry_count: 0,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            scheduled_at: Some(scheduled_at),
            started_at: None,
            completed_at: None,
            error: None,
        };
        
        let job_json = serde_json::to_string(&job)?;
        
        // Add to scheduled set with score = timestamp
        let score = scheduled_at.timestamp();
        conn.zadd(
            format!("{}:scheduled", self.queue_name),
            job_json,
            score,
        ).await?;
        
        Ok(job.id)
    }
    
    /// Dequeue next job (blocks until available)
    pub async fn dequeue(&self, timeout_secs: u64) -> Result<Option<Job>, JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Check scheduled jobs first
        self.process_scheduled_jobs().await?;
        
        // Pop from queue with timeout
        let result: Option<(String, String)> = conn
            .blpop(&self.queue_name, timeout_secs as usize)
            .await?;
        
        match result {
            Some((_, job_json)) => {
                let mut job: Job = serde_json::from_str(&job_json)?;
                job.status = JobStatus::Running;
                job.started_at = Some(Utc::now());
                
                // Update job status
                self.update_job(&job).await?;
                
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }
    
    /// Mark job as completed
    pub async fn complete(&self, job_id: Uuid) -> Result<(), JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let key = format!("job:{}", job_id);
        let job_json: Option<String> = conn.get(&key).await?;
        
        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)?;
            job.status = JobStatus::Completed;
            job.completed_at = Some(Utc::now());
            
            let updated_json = serde_json::to_string(&job)?;
            conn.set_ex(&key, updated_json, 86400).await?;
        }
        
        Ok(())
    }
    
    /// Mark job as failed and retry if possible
    pub async fn fail(&self, job_id: Uuid, error: String) -> Result<bool, JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let key = format!("job:{}", job_id);
        let job_json: Option<String> = conn.get(&key).await?;
        
        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)?;
            job.retry_count += 1;
            job.error = Some(error);
            
            if job.retry_count < job.max_retries {
                // Retry with exponential backoff
                job.status = JobStatus::Retrying;
                let backoff_secs = 2_u64.pow(job.retry_count);
                let retry_at = Utc::now() + Duration::seconds(backoff_secs as i64);
                job.scheduled_at = Some(retry_at);
                
                // Add to scheduled set
                let job_json = serde_json::to_string(&job)?;
                conn.zadd(
                    format!("{}:scheduled", self.queue_name),
                    &job_json,
                    retry_at.timestamp(),
                ).await?;
                
                conn.set_ex(&key, &job_json, 86400).await?;
                
                Ok(true) // Will retry
            } else {
                // Max retries exceeded
                job.status = JobStatus::Failed;
                job.completed_at = Some(Utc::now());
                
                let updated_json = serde_json::to_string(&job)?;
                conn.set_ex(&key, updated_json, 86400).await?;
                
                // Move to dead letter queue
                conn.rpush(
                    format!("{}:dead_letter", self.queue_name),
                    updated_json,
                ).await?;
                
                Ok(false) // Failed permanently
            }
        } else {
            Ok(false)
        }
    }
    
    /// Process scheduled jobs that are ready
    async fn process_scheduled_jobs(&self) -> Result<(), JobError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let now = Utc::now().timestamp();
        let key = format!("{}:scheduled", self.queue_name);
        
        // Get jobs scheduled before now
        let jobs: Vec<String> = conn
            .zrangebyscore_limit(&key, 0, now, 0, 100)
            .await?;
        
        for job_json in jobs {
            // Move to main queue
            conn.rpush(&self.queue_name, &job_json).await?;
            // Remove from scheduled set
            conn.zrem(&key, &job_json).await?;
        }
        
        Ok(())
    }
    
    async fn update_job(&self, job: &Job) -> Result<(), JobError> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("job:{}", job.id);
        let job_json = serde_json::to_string(job)?;
        conn.set_ex(&key, job_json, 86400).await?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
