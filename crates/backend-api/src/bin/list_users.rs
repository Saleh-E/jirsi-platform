use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = "postgres://postgres@localhost:15432/saas";
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    // 1. Update Deal Stage Options
    let sys_tenant_id = Uuid::parse_str("b128c8da-6e56-485d-b2fe-e45fb7492b2e")?;
    let deal_entity_id = Uuid::parse_str("e0000000-0000-0000-0000-000000000003")?;

    sqlx::query(
        "UPDATE field_defs SET options = $1 WHERE name = 'stage' AND entity_type_id = $2"
    )
    .bind(serde_json::json!([
        {"value": "prospecting", "label": "Prospecting", "color": "#94a3b8"},
        {"value": "qualification", "label": "Qualification", "color": "#6366f1"},
        {"value": "proposal", "label": "Proposal", "color": "#f59e0b"},
        {"value": "negotiation", "label": "Negotiation", "color": "#ec4899"},
        {"value": "closed_won", "label": "Closed Won", "color": "#22c55e"},
        {"value": "closed_lost", "label": "Closed Lost", "color": "#ef4444"}
    ]))
    .bind(deal_entity_id)
    .execute(&pool)
    .await?;

    println!("Deal stage options updated.");

    Ok(())
}
