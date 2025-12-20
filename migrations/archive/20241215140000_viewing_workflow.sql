-- Phase 3 Task P3-14: Workflow - Viewing Completion
-- Actions when viewing status changes to Completed

INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system,
    trigger_type, trigger_entity, trigger_config, conditions, actions, node_graph)
VALUES
('bf000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Viewing Completion Handler',
 'When a viewing is completed: send feedback request to viewer, log activity on contact',
 true, true,
 'field_changed', 'viewing',
 '{"field": "status", "to": "completed"}'::jsonb,
 '[{"field": "status", "operator": "equals", "value": "completed"}]'::jsonb,
 '[
   {
     "id": "step_1",
     "type": "send_notification",
     "channel": "email",
     "template": "viewing_feedback_request",
     "to": "{{viewing.contact_id}}",
     "to_type": "contact"
   },
   {
     "id": "step_2",
     "type": "log_activity",
     "entity_type": "contact",
     "entity_id": "{{viewing.contact_id}}",
     "activity_type": "viewing_completed",
     "title": "Completed viewing of {{property.title}}"
   },
   {
     "id": "step_3",
     "type": "log_activity",
     "entity_type": "property",
     "entity_id": "{{viewing.property_id}}",
     "activity_type": "viewing_completed",
     "title": "Viewing completed by {{contact.first_name}}"
   }
 ]'::jsonb,
 '{"nodes": [], "edges": []}'::jsonb)
ON CONFLICT (id) DO UPDATE SET actions = EXCLUDED.actions;

-- Feedback request template
INSERT INTO notification_templates (id, tenant_id, name, label, channel, subject, body, is_system, variables)
VALUES
('bf000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'viewing_feedback_request', 'Viewing Feedback Request', 'email',
 'How was your viewing of {{property_title}}?',
 E'Dear {{first_name}},\n\nThank you for viewing {{property_title}} with us.\n\nWe would love to hear your feedback! Please take a moment to share your thoughts:\n\n1. How would you rate the property?\n2. Does it meet your requirements?\n3. Would you like to schedule another viewing or make an offer?\n\nSimply reply to this email with your feedback.\n\nBest regards,\n{{agent_name}}\nThe Real Estate Team',
 true, '["first_name", "property_title", "agent_name"]'::jsonb)
ON CONFLICT (tenant_id, name) DO UPDATE SET subject = EXCLUDED.subject, body = EXCLUDED.body;
