-- ============================================================================
-- Contract Lifecycle Workflow
-- Manages contract states: Draft → Pending → Active → Completed/Terminated
-- ============================================================================

-- Insert Contract Lifecycle workflow definition
INSERT INTO workflow_defs (tenant_id, entity_type, name, trigger_type, trigger_config, nodes, edges, is_active, version)
SELECT 
    t.id,
    'contract',
    'Contract Lifecycle',
    'on_create',
    '{"delay_seconds": 0}'::jsonb,
    -- Workflow nodes
    '[
        {
            "id": "trigger_start",
            "type": "trigger",
            "position": {"x": 100, "y": 200},
            "data": {"label": "Contract Created"}
        },
        {
            "id": "set_draft",
            "type": "action_set_field",
            "position": {"x": 300, "y": 200},
            "data": {
                "label": "Set Status: Draft",
                "field": "status",
                "value": "draft"
            }
        },
        {
            "id": "notify_landlord",
            "type": "action_whatsapp",
            "position": {"x": 500, "y": 100},
            "data": {
                "label": "Notify Landlord",
                "template_name": "contract_created",
                "language": "en",
                "phone_field": "landlord_phone"
            }
        },
        {
            "id": "notify_tenant",
            "type": "action_whatsapp",
            "position": {"x": 500, "y": 300},
            "data": {
                "label": "Notify Tenant",
                "template_name": "contract_created",
                "language": "en",
                "phone_field": "tenant_phone"
            }
        },
        {
            "id": "create_payment_schedule",
            "type": "action_custom",
            "position": {"x": 700, "y": 200},
            "data": {
                "label": "Create Payment Schedule",
                "action": "create_payment_schedule"
            }
        }
    ]'::jsonb,
    -- Workflow edges
    '[
        {"id": "e1", "source": "trigger_start", "target": "set_draft"},
        {"id": "e2", "source": "set_draft", "target": "notify_landlord"},
        {"id": "e3", "source": "set_draft", "target": "notify_tenant"},
        {"id": "e4", "source": "notify_landlord", "target": "create_payment_schedule"},
        {"id": "e5", "source": "notify_tenant", "target": "create_payment_schedule"}
    ]'::jsonb,
    TRUE,
    1
FROM tenants t
WHERE NOT EXISTS (
    SELECT 1 FROM workflow_defs wd 
    WHERE wd.tenant_id = t.id AND wd.name = 'Contract Lifecycle'
);

-- Contract Status Change Workflow (Draft -> Pending when signed by one party)
INSERT INTO workflow_defs (tenant_id, entity_type, name, trigger_type, trigger_config, nodes, edges, is_active, version)
SELECT 
    t.id,
    'contract',
    'Contract Signature Flow',
    'on_field_change',
    '{"field": "landlord_signed", "condition": "changed_to", "value": true}'::jsonb,
    '[
        {
            "id": "trigger_signed",
            "type": "trigger",
            "position": {"x": 100, "y": 200},
            "data": {"label": "Landlord Signed"}
        },
        {
            "id": "check_tenant_signed",
            "type": "condition_if",
            "position": {"x": 300, "y": 200},
            "data": {
                "label": "Tenant Also Signed?",
                "field": "tenant_signed",
                "operator": "equals",
                "value": true
            }
        },
        {
            "id": "set_pending",
            "type": "action_set_field",
            "position": {"x": 500, "y": 100},
            "data": {
                "label": "Set Status: Pending",
                "field": "status",
                "value": "pending"
            }
        },
        {
            "id": "set_active",
            "type": "action_set_field",
            "position": {"x": 500, "y": 300},
            "data": {
                "label": "Set Status: Active",
                "field": "status",
                "value": "active"
            }
        },
        {
            "id": "notify_active",
            "type": "action_whatsapp",
            "position": {"x": 700, "y": 300},
            "data": {
                "label": "Notify Both Parties",
                "template_name": "contract_active",
                "language": "en"
            }
        },
        {
            "id": "collect_deposit",
            "type": "action_collect_payment",
            "position": {"x": 900, "y": 300},
            "data": {
                "label": "Collect Security Deposit",
                "amount_field": "security_deposit",
                "payment_type": "deposit"
            }
        }
    ]'::jsonb,
    '[
        {"id": "e1", "source": "trigger_signed", "target": "check_tenant_signed"},
        {"id": "e2", "source": "check_tenant_signed", "target": "set_pending", "sourceHandle": "false"},
        {"id": "e3", "source": "check_tenant_signed", "target": "set_active", "sourceHandle": "true"},
        {"id": "e4", "source": "set_active", "target": "notify_active"},
        {"id": "e5", "source": "notify_active", "target": "collect_deposit"}
    ]'::jsonb,
    TRUE,
    1
FROM tenants t
WHERE NOT EXISTS (
    SELECT 1 FROM workflow_defs wd 
    WHERE wd.tenant_id = t.id AND wd.name = 'Contract Signature Flow'
);

-- Contract Completion Workflow
INSERT INTO workflow_defs (tenant_id, entity_type, name, trigger_type, trigger_config, nodes, edges, is_active, version)
SELECT 
    t.id,
    'contract',
    'Contract Completion',
    'on_field_change',
    '{"field": "end_date", "condition": "is_past"}'::jsonb,
    '[
        {
            "id": "trigger_end",
            "type": "trigger",
            "position": {"x": 100, "y": 200},
            "data": {"label": "Contract End Date Reached"}
        },
        {
            "id": "check_renewed",
            "type": "condition_if",
            "position": {"x": 300, "y": 200},
            "data": {
                "label": "Contract Renewed?",
                "field": "is_renewed",
                "operator": "equals",
                "value": true
            }
        },
        {
            "id": "set_completed",
            "type": "action_set_field",
            "position": {"x": 500, "y": 100},
            "data": {
                "label": "Set Status: Completed",
                "field": "status",
                "value": "completed"
            }
        },
        {
            "id": "process_renewal",
            "type": "action_custom",
            "position": {"x": 500, "y": 300},
            "data": {
                "label": "Process Renewal",
                "action": "create_renewal_contract"
            }
        },
        {
            "id": "refund_deposit",
            "type": "action_custom",
            "position": {"x": 700, "y": 100},
            "data": {
                "label": "Process Deposit Refund",
                "action": "refund_deposit"
            }
        }
    ]'::jsonb,
    '[
        {"id": "e1", "source": "trigger_end", "target": "check_renewed"},
        {"id": "e2", "source": "check_renewed", "target": "set_completed", "sourceHandle": "false"},
        {"id": "e3", "source": "check_renewed", "target": "process_renewal", "sourceHandle": "true"},
        {"id": "e4", "source": "set_completed", "target": "refund_deposit"}
    ]'::jsonb,
    TRUE,
    1
FROM tenants t
WHERE NOT EXISTS (
    SELECT 1 FROM workflow_defs wd 
    WHERE wd.tenant_id = t.id AND wd.name = 'Contract Completion'
);
