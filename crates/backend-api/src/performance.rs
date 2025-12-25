//! Performance Configuration

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Optimized database connection pool settings
pub fn create_optimized_pool(database_url: &str) -> sqlx::PgPool {
    PgPoolOptions::new()
        // Connection pool size
        .max_connections(20)          // Max concurrent connections
        .min_connections(5)            // Keep 5 warm connections
        
        // Connection timeouts
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))  // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        
        // Connection testing
        .test_before_acquire(true)     // Validate before use
        
        // Build pool
        .connect_lazy(database_url)
        .expect("Failed to create connection pool")
}

/// Cache warming on application startup
pub async fn warm_cache(pool: &sqlx::PgPool, cache: &crate::cache::RedisCache) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Warming cache...");
    
    // 1. Preload metadata definitions
    let metadata = sqlx::query!(
        "SELECT entity_code, definition FROM entity_definitions"
    )
    .fetch_all(pool)
    .await?;
    
    for record in metadata {
        cache.set_with_ttl(
            &format!("metadata:{}", record.entity_code),
            &record.definition,
            Duration::from_secs(3600), // 1 hour
        ).await?;
    }
    
    // 2. Preload frequently accessed lookups
    let lookups = sqlx::query!(
        "SELECT entity_code, lookup_data FROM lookup_tables"
    )
    .fetch_all(pool)
    .await?;
    
    for record in lookups {
        cache.set_with_ttl(
            &format!("lookup:{}", record.entity_code),
            &record.lookup_data,
            Duration::from_secs(1800), // 30 minutes
        ).await?;
    }
    
    tracing::info!("Cache warmed with {} metadata and {} lookups", 
        metadata.len(), lookups.len());
    
    Ok(())
}

/// Performance metrics
pub struct PerformanceMetrics {
    pub request_count: std::sync::atomic::AtomicU64,
    pub avg_response_time_ms: std::sync::atomic::AtomicU64,
    pub cache_hit_rate: std::sync::atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            request_count: std::sync::atomic::AtomicU64::new(0),
            avg_response_time_ms: std::sync::atomic::AtomicU64::new(0),
            cache_hit_rate: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    pub fn record_request(&self, duration_ms: u64) {
        use std::sync::atomic::Ordering;
        
        self.request_count.fetch_add(1, Ordering::Relaxed);
        
        // Simple moving average
        let current_avg = self.avg_response_time_ms.load(Ordering::Relaxed);
        let new_avg = (current_avg + duration_ms) / 2;
        self.avg_response_time_ms.store(new_avg, Ordering::Relaxed);
    }
    
    pub fn record_cache_hit(&self, hit: bool) {
        use std::sync::atomic::Ordering;
        
        if hit {
            self.cache_hit_rate.fetch_add(1, Ordering::Relaxed);
        }
    }
}
