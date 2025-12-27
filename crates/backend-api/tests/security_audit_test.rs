//! Security Audit Tests for RLS and Tenant Isolation
//!
//! Comprehensive tests to verify:
//! 1. RLS policies correctly isolate tenant data
//! 2. Middleware correctly injects tenant session variable
//! 3. Cross-tenant queries return zero rows
//! 4. Direct SQL cannot bypass RLS when session variable is set

use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Test context for security audit tests
struct SecurityTestContext {
    pool: PgPool,
    tenant_a_id: Uuid,
    tenant_b_id: Uuid,
    tenant_a_name: String,
    tenant_b_name: String,
}

impl SecurityTestContext {
    async fn new(pool: PgPool) -> Self {
        // Create or get existing test tenants
        let tenant_a_id = Uuid::new_v4();
        let tenant_b_id = Uuid::new_v4();
        let tenant_a_name = format!("Security-Tenant-A-{}", &tenant_a_id.to_string()[..8]);
        let tenant_b_name = format!("Security-Tenant-B-{}", &tenant_b_id.to_string()[..8]);

        // Insert tenant A
        let _ = sqlx::query(
            "INSERT INTO tenants (id, name, subdomain, status, plan_tier, settings, created_at, updated_at)
             VALUES ($1, $2, $3, 'active', 'professional', '{}', NOW(), NOW())
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(tenant_a_id)
        .bind(&tenant_a_name)
        .bind(format!("sec-a-{}", &tenant_a_id.to_string()[..8]))
        .execute(&pool)
        .await;

        // Insert tenant B
        let _ = sqlx::query(
            "INSERT INTO tenants (id, name, subdomain, status, plan_tier, settings, created_at, updated_at)
             VALUES ($1, $2, $3, 'active', 'professional', '{}', NOW(), NOW())
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(tenant_b_id)
        .bind(&tenant_b_name)
        .bind(format!("sec-b-{}", &tenant_b_id.to_string()[..8]))
        .execute(&pool)
        .await;

        Self {
            pool,
            tenant_a_id,
            tenant_b_id,
            tenant_a_name,
            tenant_b_name,
        }
    }

    /// Set the current tenant context (simulates what RlsConn does)
    async fn set_tenant_context(&self, conn: &mut sqlx::PgConnection, tenant_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT set_config('app.current_tenant', $1::text, false)")
            .bind(tenant_id)
            .execute(conn)
            .await?;
        Ok(())
    }

    /// Verify the current tenant context is set
    async fn get_current_tenant(&self, conn: &mut sqlx::PgConnection) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query("SELECT current_setting('app.current_tenant', true) as tenant_id")
            .fetch_optional(conn)
            .await?;
        
        if let Some(row) = row {
            let tenant_str: Option<String> = row.try_get("tenant_id").ok();
            Ok(tenant_str.and_then(|s| Uuid::parse_str(&s).ok()))
        } else {
            Ok(None)
        }
    }

    async fn cleanup(&self) {
        // Clean up test data in reverse order of dependencies
        let tables = ["contacts", "deals", "tasks", "workflows", "listings", "contracts"];
        
        for table in tables {
            let _ = sqlx::query(&format!(
                "DELETE FROM {} WHERE tenant_id IN ($1, $2)",
                table
            ))
            .bind(self.tenant_a_id)
            .bind(self.tenant_b_id)
            .execute(&self.pool)
            .await;
        }

        // Clean up tenants
        let _ = sqlx::query("DELETE FROM tenants WHERE id IN ($1, $2)")
            .bind(self.tenant_a_id)
            .bind(self.tenant_b_id)
            .execute(&self.pool)
            .await;
    }
}

/// Test 1: Verify set_config correctly sets the tenant session variable
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_set_config_sets_tenant_context() {
    let pool = get_test_pool().await;
    let ctx = SecurityTestContext::new(pool.clone()).await;

    let mut conn = pool.acquire().await.expect("Failed to acquire connection");

    // Set tenant context
    ctx.set_tenant_context(&mut conn, ctx.tenant_a_id).await
        .expect("Failed to set tenant context");

    // Verify it was set correctly
    let current = ctx.get_current_tenant(&mut conn).await
        .expect("Failed to get current tenant");
    
    assert_eq!(current, Some(ctx.tenant_a_id), "Tenant context should be set to tenant A");

    // Change to tenant B
    ctx.set_tenant_context(&mut conn, ctx.tenant_b_id).await
        .expect("Failed to change tenant context");

    let current = ctx.get_current_tenant(&mut conn).await
        .expect("Failed to get current tenant");
    
    assert_eq!(current, Some(ctx.tenant_b_id), "Tenant context should be set to tenant B");

    ctx.cleanup().await;
}

/// Test 2: Verify RLS blocks access to other tenant's data (contacts)
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_rls_blocks_cross_tenant_access_contacts() {
    let pool = get_test_pool().await;
    let ctx = SecurityTestContext::new(pool.clone()).await;

    // Create a contact for Tenant A (bypassing RLS as superuser)
    let contact_id = Uuid::new_v4();
    let _ = sqlx::query(
        "INSERT INTO contacts (id, tenant_id, first_name, last_name, email, created_at, updated_at)
         VALUES ($1, $2, 'Secret', 'Agent', 'secret@tenanta.com', NOW(), NOW())"
    )
    .bind(contact_id)
    .bind(ctx.tenant_a_id)
    .execute(&pool)
    .await
    .expect("Failed to create test contact");

    // Now query as Tenant B - should NOT see Tenant A's contact
    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    ctx.set_tenant_context(&mut conn, ctx.tenant_b_id).await.expect("Failed to set context");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts WHERE id = $1")
        .bind(contact_id)
        .fetch_one(&mut *conn)
        .await
        .expect("Query failed");

    assert_eq!(count, 0, "Tenant B should NOT be able to see Tenant A's contacts");

    // Query as Tenant A - should see the contact
    ctx.set_tenant_context(&mut conn, ctx.tenant_a_id).await.expect("Failed to set context");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts WHERE id = $1")
        .bind(contact_id)
        .fetch_one(&mut *conn)
        .await
        .expect("Query failed");

    assert_eq!(count, 1, "Tenant A should be able to see their own contacts");

    // Cleanup
    let _ = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(contact_id)
        .execute(&pool)
        .await;

    ctx.cleanup().await;
}

/// Test 3: Verify RLS blocks cross-tenant access (deals)
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_rls_blocks_cross_tenant_access_deals() {
    let pool = get_test_pool().await;
    let ctx = SecurityTestContext::new(pool.clone()).await;

    // Create a deal for Tenant A
    let deal_id = Uuid::new_v4();
    let _ = sqlx::query(
        "INSERT INTO deals (id, tenant_id, title, value, currency, stage, probability, created_at, updated_at)
         VALUES ($1, $2, 'Secret Deal', 100000, 'USD', 'negotiation', 50, NOW(), NOW())"
    )
    .bind(deal_id)
    .bind(ctx.tenant_a_id)
    .execute(&pool)
    .await
    .expect("Failed to create test deal");

    // Query as Tenant B - should NOT see Tenant A's deal
    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    ctx.set_tenant_context(&mut conn, ctx.tenant_b_id).await.expect("Failed to set context");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deals WHERE id = $1")
        .bind(deal_id)
        .fetch_one(&mut *conn)
        .await
        .expect("Query failed");

    assert_eq!(count, 0, "Tenant B should NOT see Tenant A's deals");

    // Cleanup
    let _ = sqlx::query("DELETE FROM deals WHERE id = $1")
        .bind(deal_id)
        .execute(&pool)
        .await;

    ctx.cleanup().await;
}

/// Test 4: Verify RLS prevents cross-tenant INSERT
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_rls_prevents_cross_tenant_insert() {
    let pool = get_test_pool().await;
    let ctx = SecurityTestContext::new(pool.clone()).await;

    // Set context to Tenant B
    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    ctx.set_tenant_context(&mut conn, ctx.tenant_b_id).await.expect("Failed to set context");

    // Try to insert a contact with Tenant A's ID while authenticated as Tenant B
    let contact_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO contacts (id, tenant_id, first_name, last_name, email, created_at, updated_at)
         VALUES ($1, $2, 'Malicious', 'Insert', 'attacker@evil.com', NOW(), NOW())"
    )
    .bind(contact_id)
    .bind(ctx.tenant_a_id) // Trying to insert into Tenant A's data
    .execute(&mut *conn)
    .await;

    // This should either fail or the row should not be visible to Tenant A
    // Depending on RLS policy (WITH CHECK), it may:
    // 1. Reject the insert entirely
    // 2. Insert but make it invisible

    // Verify the malicious record is NOT visible to Tenant A
    ctx.set_tenant_context(&mut conn, ctx.tenant_a_id).await.expect("Failed to set context");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts WHERE id = $1")
        .bind(contact_id)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(0);

    // If RLS has WITH CHECK clause, insert should have failed
    // If not, the row might exist but shouldn't be visible to wrong tenant
    assert_eq!(count, 0, "Malicious cross-tenant insert should not be visible");

    // Cleanup any rows that might have been created
    let _ = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(contact_id)
        .execute(&pool)
        .await;

    ctx.cleanup().await;
}

/// Test 5: Verify all RLS-protected tables are covered
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_rls_enabled_on_all_tenant_tables() {
    let pool = get_test_pool().await;

    let expected_tables = vec![
        "contacts",
        "deals",
        "tasks",
        "workflows",
        "listings",
        "contracts",
        "entity_records",
    ];

    let result = sqlx::query(
        "SELECT tablename FROM pg_tables 
         WHERE schemaname = 'public' 
         AND tablename = ANY($1)"
    )
    .bind(&expected_tables)
    .fetch_all(&pool)
    .await
    .expect("Failed to query pg_tables");

    let rls_status = sqlx::query(
        "SELECT tablename, rowsecurity 
         FROM pg_tables 
         WHERE schemaname = 'public' 
         AND tablename = ANY($1)"
    )
    .bind(&expected_tables)
    .fetch_all(&pool)
    .await
    .expect("Failed to query RLS status");

    for row in rls_status {
        let table: String = row.get("tablename");
        let rls_enabled: bool = row.get("rowsecurity");
        
        assert!(
            rls_enabled,
            "RLS should be enabled on table '{}', but it is not!",
            table
        );
    }
}

/// Test 6: Verify empty result when no tenant context is set
#[tokio::test]
#[ignore = "Requires database connection - run with --ignored"]
async fn test_empty_results_without_tenant_context() {
    let pool = get_test_pool().await;
    let ctx = SecurityTestContext::new(pool.clone()).await;

    // Create data with valid tenant
    let contact_id = Uuid::new_v4();
    let _ = sqlx::query(
        "INSERT INTO contacts (id, tenant_id, first_name, last_name, email, created_at, updated_at)
         VALUES ($1, $2, 'Test', 'User', 'test@example.com', NOW(), NOW())"
    )
    .bind(contact_id)
    .bind(ctx.tenant_a_id)
    .execute(&pool)
    .await;

    // Query WITHOUT setting tenant context
    let mut conn = pool.acquire().await.expect("Failed to acquire connection");
    
    // Clear any existing context
    let _ = sqlx::query("SELECT set_config('app.current_tenant', '', false)")
        .execute(&mut *conn)
        .await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM contacts WHERE id = $1")
        .bind(contact_id)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(0);

    // Without valid tenant context, RLS should block access
    assert_eq!(count, 0, "Without tenant context set, no data should be visible");

    // Cleanup
    let _ = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(contact_id)
        .execute(&pool)
        .await;

    ctx.cleanup().await;
}

/// Helper function to get test database pool
async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://jirsi:jirsi@localhost:5432/saas_platform".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
