-- Agent Targets: Allow managers to set goals for agents
-- This enables "Performance vs Goals" tracking in the dashboard

CREATE TABLE agent_targets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- What metric are we tracking?
    metric_type VARCHAR(50) NOT NULL CHECK (metric_type IN (
        'revenue',
        'deals_won', 
        'calls_made',
        'leads_created',
        'viewings_completed',
        'listings_added'
    )),
    
    -- What time period?
    period_type VARCHAR(20) NOT NULL CHECK (period_type IN ('monthly', 'quarterly', 'yearly')),
    
    -- The goal value
    target_value DECIMAL(15,2) NOT NULL,
    
    -- Active period
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    -- Ensure one target per user/metric/period combo
    CONSTRAINT unique_agent_target 
        UNIQUE (tenant_id, user_id, metric_type, period_type, start_date)
);

-- Indexes for common queries
CREATE INDEX idx_agent_targets_tenant ON agent_targets(tenant_id);
CREATE INDEX idx_agent_targets_user ON agent_targets(user_id);
CREATE INDEX idx_agent_targets_date ON agent_targets(start_date, end_date);
CREATE INDEX idx_agent_targets_lookup ON agent_targets(tenant_id, user_id, metric_type, start_date);

-- Seed some example targets for the demo tenant
INSERT INTO agent_targets (tenant_id, user_id, metric_type, period_type, target_value, start_date, end_date)
SELECT 
    t.id,
    u.id,
    metric.type,
    'monthly',
    metric.value,
    DATE_TRUNC('month', CURRENT_DATE)::date,
    (DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month - 1 day')::date
FROM tenants t
CROSS JOIN users u
CROSS JOIN (
    VALUES 
        ('revenue', 50000.00),
        ('deals_won', 5.00),
        ('leads_created', 20.00)
) AS metric(type, value)
WHERE t.subdomain = 'demo'
  AND u.tenant_id = t.id
LIMIT 9; -- 3 metrics * up to 3 users
