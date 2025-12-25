//! Job Workers - Process background jobs

use super::queue::{JobQueue, Job, JobStatus};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, error};

pub type JobHandler = Arc<dyn Fn(serde_json::Value) -> BoxFuture<'static, Result<(), String>> + Send + Sync>;
type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

pub struct Worker {
    id: String,
    queue: Arc<JobQueue>,
    handlers: Arc<std::collections::HashMap<String, JobHandler>>,
}

impl Worker {
    pub fn new(
        id: String,
        queue: Arc<JobQueue>,
        handlers: std::collections::HashMap<String, JobHandler>,
    ) -> Self {
        Self {
            id,
            queue,
            handlers: Arc::new(handlers),
        }
    }
    
    /// Start worker loop
    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            info!("Worker {} started", self.id);
            
            loop {
                match self.process_next_job().await {
                    Ok(processed) => {
                        if !processed {
                            // No job available, sleep briefly
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                    Err(e) => {
                        error!("Worker {} error: {}", self.id, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        })
    }
    
    async fn process_next_job(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Dequeue with 5 second timeout
        let job = self.queue.dequeue(5).await?;
        
        let job = match job {
            Some(j) => j,
            None => return Ok(false), // No job available
        };
        
        info!("Worker {} processing job {} ({})", self.id, job.id, job.job_type);
        
        // Find handler
        let handler = match self.handlers.get(&job.job_type) {
            Some(h) => h,
            None => {
                error!("No handler for job type: {}", job.job_type);
                self.queue.fail(job.id, format!("No handler for type: {}", job.job_type)).await?;
                return Ok(true);
            }
        };
        
        // Execute job
        match handler(job.payload.clone()).await {
            Ok(()) => {
                info!("Job {} completed successfully", job.id);
                self.queue.complete(job.id).await?;
            }
            Err(e) => {
                error!("Job {} failed: {}", job.id, e);
                let will_retry = self.queue.fail(job.id, e).await?;
                if will_retry {
                    info!("Job {} will be retried", job.id);
                } else {
                    error!("Job {} failed permanently", job.id);
                }
            }
        }
        
        Ok(true)
    }
}

/// Worker pool manager
pub struct WorkerPool {
    workers: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(
        worker_count: usize,
        queue: Arc<JobQueue>,
        handlers: std::collections::HashMap<String, JobHandler>,
    ) -> Self {
        let handlers = Arc::new(handlers);
        let mut workers = Vec::new();
        
        for i in 0..worker_count {
            let worker_id = format!("worker-{}", i);
            let worker = Worker::new(
                worker_id,
                Arc::clone(&queue),
                (*handlers).clone(),
            );
            workers.push(worker.start());
        }
        
        Self { workers }
    }
    
    pub async fn shutdown(self) {
        for handle in self.workers {
            handle.abort();
        }
    }
}
