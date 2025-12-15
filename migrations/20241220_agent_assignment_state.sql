-- Agent Assignment State for Round Robin
-- Tracks assignment order to ensure fair distribution

CREATE TABLE IF NOT EXISTS agent_round_robin_state (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    last_assigned_at TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:00+00',
    assignment_count INTEGER DEFAULT 0,
    PRIMARY KEY (tenant_id, user_id)
);

-- Index for efficient queries
CREATE INDEX IF NOT EXISTS idx_agent_rr_state_tenant_last 
ON agent_round_robin_state(tenant_id, last_assigned_at ASC);

-- Initialize state for existing agents
INSERT INTO agent_round_robin_state (tenant_id, user_id)
SELECT tenant_id, id FROM users WHERE role = 'agent'
ON CONFLICT DO NOTHING;

-- Add node_graph column to workflow_defs if not exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'workflow_defs' AND column_name = 'node_graph'
    ) THEN
        ALTER TABLE workflow_defs ADD COLUMN node_graph JSONB DEFAULT '{}';
    END IF;
END $$;
