-- Event Store Schema for PostgreSQL
-- This stores the immutable event log for Event Sourcing

-- Events table - the source of truth
CREATE TABLE IF NOT EXISTS events (
    -- Identity
    sequence_number BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL DEFAULT gen_random_uuid(),
    
    -- Aggregate info
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    
    -- Event data
    event_type VARCHAR(200) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB,
    
    -- Causation tracking
    correlation_id UUID,
    causation_id UUID,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    
    -- Optimistic concurrency control
    UNIQUE(aggregate_id, aggregate_version)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_events_aggregate 
    ON events(aggregate_id, aggregate_version);

CREATE INDEX IF NOT EXISTS idx_events_type 
    ON events(event_type);

CREATE INDEX IF NOT EXISTS idx_events_created 
    ON events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_events_correlation 
    ON events(correlation_id) WHERE correlation_id IS NOT NULL;

-- Snapshots table for performance (optional but recommended)
CREATE TABLE IF NOT EXISTS snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_type 
    ON snapshots(aggregate_type);

-- Projections table (tracks last processed event per projection)
CREATE TABLE IF NOT EXISTS projection_state (
    projection_name VARCHAR(200) PRIMARY KEY,
    last_sequence_number BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Comments for documentation
COMMENT ON TABLE events IS 'Immutable event log - the source of truth for event-sourced aggregates';
COMMENT ON COLUMN events.aggregate_version IS 'Used for optimistic concurrency control';
COMMENT ON COLUMN events.correlation_id IS 'Links related events (e.g., from same user action)';
COMMENT ON COLUMN events.causation_id IS 'The event that caused this event';

COMMENT ON TABLE snapshots IS 'Cached aggregate state for performance - can be rebuilt from events';
COMMENT ON TABLE projection_state IS 'Tracks projection progress for eventual consistency';
