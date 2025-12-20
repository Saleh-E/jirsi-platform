-- Workflow Execution Tracking
-- Simple log table for workflow executions

CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    workflow_id UUID,
    workflow_name VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    error_message TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
