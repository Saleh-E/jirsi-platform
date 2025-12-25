-- Performance Optimization Indexes
-- Migration: 20241225_add_performance_indexes.sql

-- Entity records - most queried table
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_tenant_type 
  ON entity_records(tenant_id, entity_type);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_created_at 
  ON entity_records(tenant_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_updated_at 
  ON entity_records(tenant_id, updated_at DESC);

-- GIN index for JSONB field_values (enables fast JSON queries)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_field_values_gin  
  ON entity_records USING GIN (field_values);

-- Specific field lookups (common queries)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_entity_records_title
  ON entity_records((field_values->>'title')) 
  WHERE entity_type IN ('contact', 'deal', 'property');

-- Events table (event sourcing)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_aggregate_created
  ON events(aggregate_id, created_at DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_events_tenant
  ON events(tenant_id, created_at DESC) 
  WHERE tenant_id IS NOT NULL;

-- Tasks (if exists)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_due_date
  ON tasks(tenant_id, due_date) 
  WHERE deleted_at IS NULL AND status != 'completed';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_assignee
  ON tasks(assignee_id, status) 
  WHERE deleted_at IS NULL;

-- Associations (relationships)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_associations_from
  ON associations(from_entity_id, association_type);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_associations_to
  ON associations(to_entity_id, association_type);

-- Users (auth)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email
  ON users(email) WHERE deleted_at IS NULL;

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_tenant
  ON users(tenant_id) WHERE deleted_at IS NULL;

-- Sessions (performance critical)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_token
  ON sessions(session_token) WHERE expires_at > NOW();

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_user
  ON sessions(user_id) WHERE expires_at > NOW();

-- Comments for documentation
COMMENT ON INDEX idx_entity_records_field_values_gin IS 'Enables fast JSONB queries on field_values';
COMMENT ON INDEX idx_events_aggregate_created IS 'Optimizes event replay for aggregates';
COMMENT ON INDEX idx_tasks_due_date IS 'Speeds up task due date queries';
