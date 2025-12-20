-- Phase 2 Task P2-C01: Automation Trigger Framework
-- Creates schema for workflow automation triggers and actions

-- ============================================================================
-- WORKFLOW DEFINITIONS TABLE
-- Stores workflow rules and automation configurations
-- ============================================================================

CREATE TABLE IF NOT EXISTS workflow_defs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Basic info
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    is_system BOOLEAN DEFAULT false,
    
    -- Trigger configuration
    trigger_type VARCHAR(50) NOT NULL, -- 'record_created', 'record_updated', 'field_changed', 'scheduled', 'manual'
    trigger_entity VARCHAR(100), -- Entity type that triggers this workflow
    trigger_config JSONB DEFAULT '{}', -- Additional trigger configuration
    
    -- Conditions (when to run)
    conditions JSONB DEFAULT '[]', -- Array of condition objects
    
    -- Actions (what to do)
    actions JSONB DEFAULT '[]', -- Array of action objects
    
    -- Node editor data (for visual editor)
    node_graph JSONB DEFAULT '{}', -- Nodes and connections for visual editor
    
    -- Execution settings
    run_async BOOLEAN DEFAULT true,
    retry_on_failure BOOLEAN DEFAULT false,
    max_retries INTEGER DEFAULT 3,
    
    -- Audit
    last_triggered_at TIMESTAMPTZ,
    trigger_count INTEGER DEFAULT 0,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_workflow_defs_tenant ON workflow_defs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_defs_trigger ON workflow_defs(trigger_type, trigger_entity);
CREATE INDEX IF NOT EXISTS idx_workflow_defs_active ON workflow_defs(is_active) WHERE is_active = true;

-- ============================================================================
-- WORKFLOW EXECUTION LOG
-- Tracks workflow executions and results
-- ============================================================================

CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    workflow_def_id UUID NOT NULL REFERENCES workflow_defs(id),
    
    -- Trigger context
    trigger_record_id UUID,
    trigger_entity_type VARCHAR(100),
    trigger_data JSONB DEFAULT '{}',
    
    -- Execution status
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    -- Results
    result JSONB DEFAULT '{}',
    error_message TEXT,
    actions_executed INTEGER DEFAULT 0,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_workflow_executions_tenant ON workflow_executions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow ON workflow_executions(workflow_def_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_created ON workflow_executions(created_at DESC);

-- ============================================================================
-- SEED WORKFLOW TEMPLATES
-- ============================================================================

INSERT INTO workflow_defs (id, tenant_id, name, description, is_system, trigger_type, trigger_entity, conditions, actions)
VALUES
-- Auto-assign viewing status
('af000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Auto-confirm Viewing Reminder', 
 'Send reminder 24h before scheduled viewing',
 true, 'scheduled', 'viewing',
 '[{"field": "scheduled_at", "operator": "in_hours", "value": 24}]'::jsonb,
 '[{"type": "send_notification", "channel": "email", "template": "viewing_reminder"}]'::jsonb),

-- Property status change notification
('af000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Property Status Change Alert',
 'Notify agent when property status changes',
 true, 'field_changed', 'property',
 '[{"field": "status"}]'::jsonb,
 '[{"type": "send_notification", "channel": "email", "template": "property_status_changed", "to_role": "agent"}]'::jsonb),

-- Offer submitted notification
('af000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Offer Submitted Notification',
 'Notify property owner when offer is submitted',
 true, 'record_created', 'offer',
 '[]'::jsonb,
 '[{"type": "send_notification", "channel": "email", "template": "offer_submitted", "to_role": "owner"},
   {"type": "send_notification", "channel": "whatsapp", "template": "offer_submitted_sms", "to_role": "owner"}]'::jsonb),

-- Contract signed workflow
('af000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Contract Signed Actions',
 'Update property status and notify parties when contract is signed',
 true, 'field_changed', 'contract',
 '[{"field": "status", "operator": "equals", "value": "active"}]'::jsonb,
 '[{"type": "update_record", "entity": "property", "relation": "property_id", "set": {"status": "sold"}},
   {"type": "send_notification", "channel": "email", "template": "contract_signed", "to_roles": ["buyer", "seller", "agent"]}]'::jsonb),

-- Deal won celebration
('af000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Deal Won Celebration',
 'Notify team when deal is won',
 true, 'field_changed', 'deal',
 '[{"field": "stage", "operator": "equals", "value": "closed_won"}]'::jsonb,
 '[{"type": "send_notification", "channel": "email", "template": "deal_won", "to": "team"}]'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- NOTIFICATION TEMPLATES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Template info
    name VARCHAR(100) NOT NULL,
    label VARCHAR(200) NOT NULL,
    channel VARCHAR(50) NOT NULL, -- 'email', 'sms', 'whatsapp', 'push'
    
    -- Content
    subject VARCHAR(500), -- For email
    body TEXT NOT NULL,
    
    -- Template variables (for UI hints)
    variables JSONB DEFAULT '[]',
    
    -- System
    is_system BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, name)
);

-- Seed notification templates
INSERT INTO notification_templates (id, tenant_id, name, label, channel, subject, body, is_system, variables)
VALUES
('af000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_reminder', 'Viewing Reminder', 'email',
 'Reminder: Property Viewing Tomorrow',
 'Hello {{contact.first_name}},\n\nThis is a reminder that you have a property viewing scheduled for tomorrow:\n\nProperty: {{property.title}}\nTime: {{viewing.scheduled_at}}\nAddress: {{property.address}}\n\nPlease confirm your attendance by replying to this email.\n\nBest regards,\n{{agent.first_name}}',
 true, '["contact.first_name", "property.title", "viewing.scheduled_at", "property.address", "agent.first_name"]'::jsonb),

('af000002-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_submitted', 'Offer Submitted', 'email',
 'New Offer Received for {{property.title}}',
 'Hello {{owner.first_name}},\n\nGreat news! A new offer has been submitted for your property:\n\nProperty: {{property.title}}\nOffer Amount: {{offer.offer_amount}} {{offer.currency}}\nFrom: {{buyer.first_name}} {{buyer.last_name}}\n\nPlease log in to review the offer details.\n\nBest regards,\nYour Real Estate Team',
 true, '["owner.first_name", "property.title", "offer.offer_amount", "buyer.first_name"]'::jsonb),

('af000002-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_submitted_sms', 'Offer Submitted (WhatsApp)', 'whatsapp',
 NULL,
 '🏠 New Offer: {{offer.offer_amount}} {{offer.currency}} received for {{property.title}}. Check your email for details.',
 true, '["offer.offer_amount", "property.title"]'::jsonb),

('af000002-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'contract_signed', 'Contract Signed', 'email',
 'Contract Signed - {{property.title}}',
 'Hello,\n\nThe contract for {{property.title}} has been signed.\n\nContract Number: {{contract.contract_number}}\nAmount: {{contract.amount}} {{contract.currency}}\nStart Date: {{contract.start_date}}\n\nCongratulations on the successful transaction!\n\nBest regards,\nYour Real Estate Team',
 true, '["property.title", "contract.contract_number", "contract.amount"]'::jsonb),

('af000002-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'deal_won', 'Deal Won', 'email',
 '🎉 Deal Won: {{deal.name}}',
 'Congratulations team!\n\nWe have successfully closed the deal:\n\nDeal: {{deal.name}}\nAmount: {{deal.amount}}\n\nGreat work everyone!',
 true, '["deal.name", "deal.amount"]'::jsonb)
ON CONFLICT (tenant_id, name) DO NOTHING;
