use sqlx::postgres::PgPoolOptions;
use std::time::Duration; // force recompile 22

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to 'postgres' database to create the new one
    let database_url = "postgres://postgres:postgres@localhost:15433/postgres";
    println!("Connecting to 'postgres' DB to create target DB...");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    // Check if exists
    let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = 'saas_platform')")
        .fetch_one(&pool)
        .await?;

    if exists.0 {
        println!("Dropping existing 'saas_platform' database...");
        sqlx::query("DROP DATABASE saas_platform WITH (FORCE)").execute(&pool).await?;
    }

    println!("Creating database 'saas_platform'...");
    sqlx::query("CREATE DATABASE saas_platform").execute(&pool).await?;
    
    // Reconnect to the new DB
    let database_url = "postgres://postgres:postgres@localhost:15433/saas_platform";
    let pool = PgPoolOptions::new().max_connections(1).connect(database_url).await?;

    println!("Running migrations...");
    match sqlx::migrate!("../../migrations").run(&pool).await {
        Ok(_) => println!("Migrations complete"),
        Err(e) => {
            println!("Migration failed: {}", e);
            // Don't exit, maybe we can inspect DB
        }
    }

    // The original instruction's `Code Edit` block had a syntax error here.
    // Assuming the intent was to use the `pool` connected to `saas_platform`
    // for the entity_types count, and the "Reconnecting to 'saas_platform'..."
    // print was meant to be removed or was a leftover from the previous structure.
    // Given the instruction "Revert to standard migration run", the system tenant
    // creation is removed, and the `pool` variable is now correctly pointing
    // to `saas_platform` after the migration.
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM entity_types")
        .fetch_one(&pool) // Use the `pool` that is now connected to `saas_platform`
        .await?;
    println!("Entity Types Count: {}", count.0);
    
    Ok(())
}
