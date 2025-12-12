//! Job processing

use sqlx::PgPool;
use uuid::Uuid;

/// Represents a job from the queue
struct Job {
    id: Uuid,
    job_type: String,
    payload: serde_json::Value,
    tenant_id: Uuid,
}

/// Process pending jobs from the queue
pub async fn process_pending_jobs(pool: &PgPool) -> anyhow::Result<u32> {
    use sqlx::Row;
    
    // Fetch pending jobs
    let rows = sqlx::query(
        r#"
        SELECT id, job_type, payload, tenant_id
        FROM job_queue
        WHERE status = 'pending'
        ORDER BY created_at
        LIMIT 10
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_all(pool)
    .await?;

    let jobs: Vec<Job> = rows
        .iter()
        .map(|row| Job {
            id: row.try_get("id").unwrap_or_default(),
            job_type: row.try_get("job_type").unwrap_or_default(),
            payload: row.try_get("payload").unwrap_or_default(),
            tenant_id: row.try_get("tenant_id").unwrap_or_default(),
        })
        .collect();

    let count = jobs.len() as u32;

    for job in jobs {
        // Mark as processing
        sqlx::query(
            r#"UPDATE job_queue SET status = 'processing', started_at = NOW() WHERE id = $1"#,
        )
        .bind(job.id)
        .execute(pool)
        .await?;

        // Process based on job type
        let result = match job.job_type.as_str() {
            "node_graph_execution" => process_node_graph_job(pool, &job.payload).await,
            "send_email" => process_email_job(pool, &job.payload).await,
            _ => {
                tracing::warn!("Unknown job type: {}", job.job_type);
                Ok(())
            }
        };

        // Update job status
        match result {
            Ok(()) => {
                sqlx::query(
                    r#"UPDATE job_queue SET status = 'completed', completed_at = NOW() WHERE id = $1"#,
                )
                .bind(job.id)
                .execute(pool)
                .await?;
            }
            Err(e) => {
                sqlx::query(
                    r#"UPDATE job_queue SET status = 'failed', error = $1, completed_at = NOW() WHERE id = $2"#,
                )
                .bind(e.to_string())
                .bind(job.id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(count)
}

async fn process_node_graph_job(_pool: &PgPool, payload: &serde_json::Value) -> anyhow::Result<()> {
    tracing::info!("Processing node graph job: {:?}", payload);
    // TODO: Execute node graph
    Ok(())
}

async fn process_email_job(_pool: &PgPool, payload: &serde_json::Value) -> anyhow::Result<()> {
    tracing::info!("Processing email job: {:?}", payload);
    // TODO: Send email
    Ok(())
}
