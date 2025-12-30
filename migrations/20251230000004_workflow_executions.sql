-- ============================================================================
-- Workflow Executions - Durable Workflow State Persistence
-- Enables workflow suspend/resume, retry, and execution history
-- ============================================================================

-- Workflow execution states
CREATE TYPE workflow_execution_status AS ENUM (
    'pending',      -- Queued but not started
    'running',      -- Currently executing
    'suspended',    -- Waiting for external event (payment, approval)
    'completed',    -- Successfully finished
    'failed',       -- Failed with error
    'cancelled',    -- Manually cancelled
    'retrying'      -- Scheduled for retry
);

-- Main workflow executions table
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Workflow definition
    workflow_id UUID NOT NULL,  -- References workflow_defs
    workflow_version INTEGER NOT NULL DEFAULT 1,
    
    -- Trigger context
    trigger_entity_type VARCHAR(100) NOT NULL,
    trigger_entity_id UUID NOT NULL,
    trigger_type VARCHAR(50) NOT NULL,  -- 'on_create', 'on_update', 'on_field_change', 'scheduled', 'webhook'
    trigger_data JSONB NOT NULL DEFAULT '{}',
    
    -- Execution state
    status workflow_execution_status NOT NULL DEFAULT 'pending',
    current_node_id VARCHAR(255),  -- Current node being executed
    
    -- Resume/suspend handling
    suspend_reason TEXT,
    suspend_data JSONB,  -- Data needed to resume
    resume_token UUID,   -- Token to resume execution
    resume_at TIMESTAMPTZ,  -- Scheduled resume time
    
    -- Execution context
    context_data JSONB NOT NULL DEFAULT '{}',  -- Accumulated workflow data
    
    -- Loop protection
    loop_count INTEGER NOT NULL DEFAULT 0,
    max_loops INTEGER NOT NULL DEFAULT 100,
    
    -- Retry handling
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    last_error TEXT,
    
    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_executions_tenant 
    ON workflow_executions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status
    ON workflow_executions(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_entity 
    ON workflow_executions(tenant_id, trigger_entity_type, trigger_entity_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_resume 
    ON workflow_executions(resume_at) 
    WHERE status = 'suspended' AND resume_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_workflow_executions_retry 
    ON workflow_executions(next_retry_at) 
    WHERE status = 'retrying';

-- Workflow execution steps (individual node executions)
CREATE TABLE IF NOT EXISTS workflow_execution_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    
    -- Node info
    node_id VARCHAR(255) NOT NULL,
    node_type VARCHAR(100) NOT NULL,
    
    -- Step state
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'skipped')),
    
    -- Input/Output
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,
    
    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_steps_execution 
    ON workflow_execution_steps(execution_id, created_at);

-- ============================================================================
-- Circuit Breaker State
-- Prevents cascading failures and enforces rate limits
-- ============================================================================

CREATE TABLE IF NOT EXISTS circuit_breakers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Circuit identifier
    circuit_key VARCHAR(255) NOT NULL,  -- e.g., 'email:sendgrid', 'sms:twilio', 'webhook:stripe'
    
    -- State
    state VARCHAR(20) NOT NULL DEFAULT 'closed'
        CHECK (state IN ('closed', 'open', 'half_open')),
    
    -- Failure tracking
    failure_count INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    last_failure_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    
    -- Configuration
    failure_threshold INTEGER NOT NULL DEFAULT 5,  -- Failures before opening
    success_threshold INTEGER NOT NULL DEFAULT 2,  -- Successes in half-open to close
    timeout_seconds INTEGER NOT NULL DEFAULT 60,   -- Time before half-open attempt
    
    -- Rate limiting
    requests_per_minute INTEGER NOT NULL DEFAULT 60,
    current_minute_count INTEGER NOT NULL DEFAULT 0,
    current_minute_start TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, circuit_key)
);

CREATE INDEX IF NOT EXISTS idx_circuit_breakers_lookup 
    ON circuit_breakers(tenant_id, circuit_key);

-- ============================================================================
-- Notification Log (for SMS, Email, WhatsApp audit)
-- ============================================================================

CREATE TABLE IF NOT EXISTS notification_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Notification details
    channel VARCHAR(50) NOT NULL CHECK (channel IN ('email', 'sms', 'whatsapp', 'push')),
    recipient VARCHAR(255) NOT NULL,  -- Email or phone number
    subject TEXT,
    body TEXT,
    template_id VARCHAR(100),
    
    -- Context
    entity_type VARCHAR(100),
    entity_id UUID,
    workflow_execution_id UUID REFERENCES workflow_executions(id) ON DELETE SET NULL,
    
    -- Provider details
    provider VARCHAR(100) NOT NULL,  -- 'sendgrid', 'twilio', 'meta_whatsapp'
    provider_message_id VARCHAR(255),
    
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'sent', 'delivered', 'failed', 'bounced', 'opened', 'clicked')),
    error_message TEXT,
    
    -- Timing
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    opened_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notification_log_tenant 
    ON notification_log(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notification_log_entity 
    ON notification_log(tenant_id, entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_notification_log_status 
    ON notification_log(tenant_id, channel, status);

-- ============================================================================
-- Triggers
-- ============================================================================

CREATE OR REPLACE FUNCTION update_workflow_execution_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    
    -- Calculate duration if completed
    IF NEW.status IN ('completed', 'failed', 'cancelled') AND NEW.completed_at IS NULL THEN
        NEW.completed_at = NOW();
        IF NEW.started_at IS NOT NULL THEN
            NEW.duration_ms = EXTRACT(EPOCH FROM (NOW() - NEW.started_at)) * 1000;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_workflow_executions_updated
    BEFORE UPDATE ON workflow_executions
    FOR EACH ROW
    EXECUTE FUNCTION update_workflow_execution_timestamp();
