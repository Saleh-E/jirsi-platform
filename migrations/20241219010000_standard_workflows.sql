-- Phase 3: Standard Workflows and Agent Stats
-- Seeds the "Bible" workflows for CRM and Real Estate automation

-- Use our demo tenant
DO $$
DECLARE
    demo_tenant_id UUID := 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
    contact_entity_id UUID;
    deal_entity_id UUID;
    offer_entity_id UUID;
    property_entity_id UUID;
    task_entity_id UUID;
    user_entity_id UUID;
BEGIN

-- Get entity type IDs
SELECT id INTO contact_entity_id FROM entity_types WHERE tenant_id = demo_tenant_id AND name = 'contact' LIMIT 1;
SELECT id INTO deal_entity_id FROM entity_types WHERE tenant_id = demo_tenant_id AND name = 'deal' LIMIT 1;
SELECT id INTO offer_entity_id FROM entity_types WHERE tenant_id = demo_tenant_id AND name = 'offer' LIMIT 1;
SELECT id INTO property_entity_id FROM entity_types WHERE tenant_id = demo_tenant_id AND name = 'property' LIMIT 1;
SELECT id INTO task_entity_id FROM entity_types WHERE tenant_id = demo_tenant_id AND name = 'task' LIMIT 1;

-- ============================================
-- WORKFLOW 1: New Lead Intake (CRM)
-- Trigger: On Contact Created
-- Actions: Create Task, Update Contact status
-- ============================================

INSERT INTO workflows (id, tenant_id, name, trigger_type, trigger_entity, trigger_config, conditions, actions, is_active, created_at, updated_at)
VALUES (
    'a0000001-0000-0000-0000-000000000001',
    demo_tenant_id,
    'New Lead Intake',
    'record_created',
    'contact',
    '{"entity_type": "contact"}'::jsonb,
    '[]'::jsonb,
    '[
        {
            "id": "create_task_1",
            "type": "create_record",
            "entity_type": "task",
            "data": {
                "title": "Call new lead",
                "status": "open",
                "priority": "high",
                "due_date": "{{now + 10 minutes}}"
            }
        },
        {
            "id": "update_status_1",
            "type": "update_record",
            "field": "lifecycle_stage",
            "value": "new"
        }
    ]'::jsonb,
    true,
    NOW(),
    NOW()
) ON CONFLICT (id) DO NOTHING;

-- ============================================
-- WORKFLOW 2: Deal Won (CRM)
-- Trigger: On Deal Updated
-- Condition: stage changed to "Won"
-- Action: Create Task for contract preparation
-- ============================================

INSERT INTO workflows (id, tenant_id, name, trigger_type, trigger_entity, trigger_config, conditions, actions, is_active, created_at, updated_at)
VALUES (
    'a0000001-0000-0000-0000-000000000002',
    demo_tenant_id,
    'Deal Won',
    'field_changed',
    'deal',
    '{"entity_type": "deal", "field": "stage"}'::jsonb,
    '[
        {
            "field": "stage",
            "operator": "changed_to",
            "value": "won"
        }
    ]'::jsonb,
    '[
        {
            "id": "create_contract_task",
            "type": "create_record",
            "entity_type": "task",
            "data": {
                "title": "Prepare Contract",
                "status": "open",
                "priority": "high",
                "assignee_id": "{{record.owner_id}}"
            }
        }
    ]'::jsonb,
    true,
    NOW(),
    NOW()
) ON CONFLICT (id) DO NOTHING;

-- ============================================
-- WORKFLOW 3: Offer Accepted (Real Estate)
-- Trigger: On Offer Updated
-- Condition: status = "Accepted"
-- Actions: Update Property to "Under Offer", Update Deal to "Negotiation"
-- ============================================

INSERT INTO workflows (id, tenant_id, name, trigger_type, trigger_entity, trigger_config, conditions, actions, is_active, created_at, updated_at)
VALUES (
    'a0000001-0000-0000-0000-000000000003',
    demo_tenant_id,
    'Offer Accepted',
    'field_changed',
    'offer',
    '{"entity_type": "offer", "field": "status"}'::jsonb,
    '[
        {
            "field": "status",
            "operator": "changed_to",
            "value": "accepted"
        }
    ]'::jsonb,
    '[
        {
            "id": "update_property",
            "type": "update_related",
            "related_entity": "property",
            "related_field": "property_id",
            "updates": {
                "status": "under_offer"
            }
        },
        {
            "id": "update_deal",
            "type": "update_related",
            "related_entity": "deal",
            "related_field": "deal_id",
            "updates": {
                "stage": "negotiation"
            }
        }
    ]'::jsonb,
    true,
    NOW(),
    NOW()
) ON CONFLICT (id) DO NOTHING;

-- ============================================
-- PART 4: Agent Daily Stats Entity
-- Unified Agent model - User with stats tracking
-- ============================================

-- Create agent_daily_stats EntityType (if not exists)
INSERT INTO entity_types (
    id, tenant_id, app_id, name, label, 
    has_activities, has_tasks, has_custom_fields, show_in_nav, is_searchable,
    created_at, updated_at
)
VALUES (
    'e0000001-0000-0000-0000-000000000100',
    demo_tenant_id,
    'crm',
    'agent_daily_stats',
    'Agent Daily Stats',
    false, false, false, false, false,
    NOW(), NOW()
) ON CONFLICT (tenant_id, name) DO NOTHING;

-- Agent Stats Fields
INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, sort_order, is_required, in_list_view, is_searchable, created_at, updated_at)
VALUES 
    -- user_id (Link to User)
    ('f0000100-0000-0000-0000-000000000001', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'user_id', 'Agent', 
     '{"type": "Link", "config": {"target_entity": "user"}}'::jsonb, 1, true, true, false, NOW(), NOW()),
    
    -- date (Date)
    ('f0000100-0000-0000-0000-000000000002', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'date', 'Date',
     '{"type": "Date"}'::jsonb, 2, true, true, false, NOW(), NOW()),
    
    -- calls_count (Number)
    ('f0000100-0000-0000-0000-000000000003', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'calls_count', 'Calls Made',
     '{"type": "Number", "config": {"decimals": 0}}'::jsonb, 3, false, true, false, NOW(), NOW()),
    
    -- deals_won (Number)
    ('f0000100-0000-0000-0000-000000000004', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'deals_won', 'Deals Won',
     '{"type": "Number", "config": {"decimals": 0}}'::jsonb, 4, false, true, false, NOW(), NOW()),
    
    -- viewings_completed (Number)
    ('f0000100-0000-0000-0000-000000000005', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'viewings_completed', 'Viewings Completed',
     '{"type": "Number", "config": {"decimals": 0}}'::jsonb, 5, false, true, false, NOW(), NOW()),
    
    -- commission_earned (Money)
    ('f0000100-0000-0000-0000-000000000006', demo_tenant_id, 'e0000001-0000-0000-0000-000000000100', 'commission_earned', 'Commission Earned',
     '{"type": "Money", "config": {"currency_code": "USD"}}'::jsonb, 6, false, true, false, NOW(), NOW())

ON CONFLICT (tenant_id, entity_type_id, name) DO NOTHING;

-- Create default view for Agent Stats
INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, created_at, updated_at)
VALUES (
    'v0000100-0000-0000-0000-000000000001',
    demo_tenant_id,
    'e0000001-0000-0000-0000-000000000100',
    'agent_performance',
    'Agent Performance',
    'table',
    true,
    true,
    '[
        {"field": "user_id", "width": null, "visible": true, "sort_order": 1},
        {"field": "date", "width": null, "visible": true, "sort_order": 2},
        {"field": "calls_count", "width": null, "visible": true, "sort_order": 3},
        {"field": "deals_won", "width": null, "visible": true, "sort_order": 4},
        {"field": "commission_earned", "width": null, "visible": true, "sort_order": 5}
    ]'::jsonb,
    NOW(),
    NOW()
) ON CONFLICT (tenant_id, entity_type_id, name) DO NOTHING;

END $$;

-- Create agent_daily_stats table (if not exists)
CREATE TABLE IF NOT EXISTS agent_daily_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    date DATE NOT NULL,
    calls_count INTEGER DEFAULT 0,
    deals_won INTEGER DEFAULT 0,
    viewings_completed INTEGER DEFAULT 0,
    commission_earned DECIMAL(12,2) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One stat record per user per day
    UNIQUE (tenant_id, user_id, date)
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_agent_daily_stats_user_date 
ON agent_daily_stats (tenant_id, user_id, date DESC);
