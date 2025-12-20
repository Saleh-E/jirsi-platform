-- Phase 3 Task P3-12: Workflow - Property Enquiry
-- Automated workflow for property enquiry form submission

-- ============================================================================
-- ENQUIRY WORKFLOW DEFINITION
-- ============================================================================

INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system,
    trigger_type, trigger_entity, trigger_config, conditions, actions, node_graph)
VALUES
('bf000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Property Enquiry Handler',
 'When a property enquiry is submitted: create/update Contact, create Deal, assign agent, create follow-up task, send acknowledgment',
 true, true,
 'form_submitted', 'enquiry',
 '{"form_type": "property_enquiry"}'::jsonb,
 '[]'::jsonb,
 '[
   {
     "id": "step_1",
     "type": "upsert_record",
     "entity": "contact",
     "match_fields": ["email"],
     "set_fields": {
       "first_name": "{{form.first_name}}",
       "last_name": "{{form.last_name}}",
       "email": "{{form.email}}",
       "phone": "{{form.phone}}",
       "lifecycle_stage": "lead"
     },
     "output_var": "contact"
   },
   {
     "id": "step_2",
     "type": "create_record",
     "entity": "deal",
     "set_fields": {
       "name": "Enquiry: {{property.title}}",
       "stage": "new",
       "contact_id": "{{contact.id}}",
       "property_id": "{{form.property_id}}"
     },
     "output_var": "deal"
   },
   {
     "id": "step_3",
     "type": "assign_agent",
     "method": "property_agent",
     "fallback": "round_robin",
     "property_id": "{{form.property_id}}",
     "output_var": "agent"
   },
   {
     "id": "step_4",
     "type": "create_record",
     "entity": "task",
     "set_fields": {
       "title": "Follow up on enquiry: {{property.title}}",
       "description": "Contact {{contact.first_name}} regarding their enquiry for {{property.title}}",
       "due_date": "{{now + 24h}}",
       "assigned_to": "{{agent.id}}",
       "related_to_type": "deal",
       "related_to_id": "{{deal.id}}",
       "priority": "high"
     }
   },
   {
     "id": "step_5",
     "type": "send_notification",
     "channel": "email",
     "template": "enquiry_acknowledgment",
     "to": "{{contact.email}}",
     "variables": {
       "first_name": "{{contact.first_name}}",
       "property_title": "{{property.title}}",
       "agent_name": "{{agent.first_name}}"
     }
   },
   {
     "id": "step_6",
     "type": "log_activity",
     "entity_type": "property",
     "entity_id": "{{form.property_id}}",
     "activity_type": "enquiry",
     "title": "New enquiry from {{contact.first_name}} {{contact.last_name}}"
   }
 ]'::jsonb,
 '{
   "nodes": [
     {"id": "trigger", "type": "trigger", "x": 100, "y": 100, "label": "Form Submitted"},
     {"id": "upsert_contact", "type": "action", "x": 300, "y": 100, "label": "Create/Update Contact"},
     {"id": "create_deal", "type": "action", "x": 500, "y": 100, "label": "Create Deal"},
     {"id": "assign_agent", "type": "action", "x": 700, "y": 100, "label": "Assign Agent"},
     {"id": "create_task", "type": "action", "x": 500, "y": 250, "label": "Create Task"},
     {"id": "send_email", "type": "action", "x": 700, "y": 250, "label": "Send Email"},
     {"id": "log_activity", "type": "action", "x": 900, "y": 175, "label": "Log Activity"}
   ],
   "edges": [
     {"from": "trigger", "to": "upsert_contact"},
     {"from": "upsert_contact", "to": "create_deal"},
     {"from": "create_deal", "to": "assign_agent"},
     {"from": "assign_agent", "to": "create_task"},
     {"from": "assign_agent", "to": "send_email"},
     {"from": "create_task", "to": "log_activity"},
     {"from": "send_email", "to": "log_activity"}
   ]
 }'::jsonb)
ON CONFLICT (id) DO UPDATE SET
    actions = EXCLUDED.actions,
    node_graph = EXCLUDED.node_graph;

-- ============================================================================
-- ENQUIRY ACKNOWLEDGMENT EMAIL TEMPLATE
-- ============================================================================

INSERT INTO notification_templates (id, tenant_id, name, label, channel, subject, body, is_system, variables)
VALUES
('bf000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'enquiry_acknowledgment', 'Enquiry Acknowledgment', 'email',
 'Thank you for your enquiry about {{property_title}}',
 E'Dear {{first_name}},\n\nThank you for your interest in {{property_title}}.\n\nWe have received your enquiry and {{agent_name}} will be in touch within 24 hours to discuss your requirements.\n\nIn the meantime, if you have any questions, please don''t hesitate to reply to this email.\n\nBest regards,\nThe Real Estate Team',
 true,
 '["first_name", "property_title", "agent_name"]'::jsonb)
ON CONFLICT (tenant_id, name) DO UPDATE SET
    subject = EXCLUDED.subject,
    body = EXCLUDED.body;
