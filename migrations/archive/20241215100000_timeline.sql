-- Phase 3 Task P3-10: Timeline Component
-- Activity timeline tables and views

-- ============================================================================
-- ACTIVITY LOG TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Activity info
    activity_type VARCHAR(100) NOT NULL, -- 'note', 'call', 'email', 'viewing', 'offer', 'task', 'status_change'
    title VARCHAR(500),
    description TEXT,
    
    -- Related entities
    entity_type VARCHAR(100), -- 'contact', 'property', 'deal', etc.
    entity_id UUID,
    
    -- Secondary entity (for links)
    related_entity_type VARCHAR(100),
    related_entity_id UUID,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- User who performed the action
    performed_by UUID REFERENCES users(id),
    
    -- Timestamp
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_activity_log_tenant ON activity_log(tenant_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_entity ON activity_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_type ON activity_log(activity_type);
CREATE INDEX IF NOT EXISTS idx_activity_log_occurred ON activity_log(occurred_at DESC);

-- ============================================================================
-- AGGREGATE TIMELINE VIEW
-- Start with activity log, can extend with triggers to populate from other entities
-- ============================================================================

CREATE OR REPLACE VIEW v_entity_timeline AS
SELECT 
    al.id,
    al.tenant_id,
    al.activity_type as event_type,
    al.title,
    al.description,
    al.entity_type,
    al.entity_id,
    al.metadata,
    al.performed_by,
    al.occurred_at,
    'activity' as source
FROM activity_log al;

-- Note: Additional entity events (viewings, offers, tasks) can be logged to activity_log
-- via triggers when those records are created/updated, ensuring consistent schema.
