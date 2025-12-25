//! Redis Cache Layer
//! 
//! Automatic caching with TTL and invalidation on events

use redis::{Client, AsyncCommands, RedisError};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

pub struct RedisCache {
    client: Client,
    default_ttl: Duration,
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        
        // Test connection
        let mut conn = client.get_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        Ok(Self {
            client,
            default_ttl: Duration::from_secs(300), // 5 minutes default
        })
    }
    
    /// Get cached value
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(json) => {
                let parsed = serde_json::from_str(&json)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }
    
    /// Set cached value with default TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), CacheError> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }
    
    /// Set cached value with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let json = serde_json::to_string(value)?;
        let ttl_secs = ttl.as_secs() as usize;
        
        conn.set_ex::<_, _, ()>(key, json, ttl_secs as u64).await?;
        
        Ok(())
    }
    
    /// Delete cached value
    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del::<_, ()>(key).await?;
        Ok(())
    }
    
    /// Delete all keys matching pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<usize, CacheError> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Get all keys matching pattern
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if keys.is_empty() {
            return Ok(0);
        }
        
        // Delete all matching keys
        let count: usize = conn.del(&keys).await?;
        
        Ok(count)
    }
    
    /// Get or compute a value
    pub async fn get_or_compute<T, F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> Result<T, CacheError>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, CacheError>>,
    {
        // Try cache first
        if let Some(cached) = self.get(key).await? {
            return Ok(cached);
        }
        
        // Compute value
        let value = compute().await?;
        
        // Cache for next time
        self.set(key, &value).await?;
        
        Ok(value)
    }
    
    /// Invalidate cache for a deal
    pub async fn invalidate_deal(&self, deal_id: uuid::Uuid) -> Result<(), CacheError> {
        // Delete specific deal cache
        self.delete(&format!("deal:{}", deal_id)).await?;
        
        // Delete deal lists that might include this deal
        self.delete_pattern("deals:*").await?;
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Computation error: {0}")]
    Computation(String),
}
