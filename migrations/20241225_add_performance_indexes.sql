-- Performance Optimization Indexes
-- Migration: 20241225_add_performance_indexes.sql
-- Fixed: References correct column names

-- Entity records - most queried table
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_tenant_type 
  ON entity_records(tenant_id, entity_type_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_created_at 
  ON entity_records(tenant_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_updated_at 
  ON entity_records(tenant_id, updated_at DESC);

-- GIN index for JSONB data field (enables fast JSON queries)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_data_gin  
  ON entity_records USING GIN (data);

-- Events table (event sourcing)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_aggregate_created
  ON events(aggregate_id, created_at DESC);

-- Tasks indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_due_date
  ON tasks(tenant_id, due_date) 
  WHERE status != 'completed';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_assignee_status
  ON tasks(assignee_id, status);

-- Associations (relationships)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_associations_source_id
  ON associations(source_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_associations_target_id
  ON associations(target_id);

-- Users (auth)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_tenant_email
  ON users(tenant_id, email);

-- Sessions (performance critical)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_token_hash_active
  ON sessions(token_hash) WHERE expires_at > NOW();

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_user_active
  ON sessions(user_id) WHERE expires_at > NOW();

-- Contacts
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_contacts_lifecycle
  ON contacts(tenant_id, lifecycle_stage);

-- Deals
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_deals_pipeline_stage
  ON deals(pipeline_id, stage);

-- Comments for documentation
COMMENT ON INDEX idx_entity_records_data_gin IS 'Enables fast JSONB queries on entity data';
COMMENT ON INDEX idx_events_aggregate_created IS 'Optimizes event replay for aggregates';
COMMENT ON INDEX idx_tasks_due_date IS 'Speeds up task due date queries';
