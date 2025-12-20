-- Phase 3 Task P3-13: Workflow - Offer Acceptance
-- Automated actions when offer status changes to Accepted

INSERT INTO workflow_defs (id, tenant_id, name, description, is_active, is_system,
    trigger_type, trigger_entity, trigger_config, conditions, actions, node_graph)
VALUES
('bf000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'Offer Acceptance Handler',
 'When an offer is accepted: update property status, update deal stage, create contract draft, notify all parties',
 true, true,
 'field_changed', 'offer',
 '{"field": "status", "to": "accepted"}'::jsonb,
 '[{"field": "status", "operator": "equals", "value": "accepted"}]'::jsonb,
 '[
   {
     "id": "step_1",
     "type": "update_record",
     "entity": "property",
     "record_id": "{{offer.property_id}}",
     "set_fields": {
       "status": "under_offer"
     }
   },
   {
     "id": "step_2",
     "type": "update_record",
     "entity": "deal",
     "find_by": {"property_id": "{{offer.property_id}}"},
     "set_fields": {
       "stage": "negotiation"
     }
   },
   {
     "id": "step_3",
     "type": "create_record",
     "entity": "contract",
     "set_fields": {
       "property_id": "{{offer.property_id}}",
       "buyer_contact_id": "{{offer.contact_id}}",
       "contract_type": "sale",
       "contract_value": "{{offer.offer_price}}",
       "status": "draft"
     },
     "output_var": "contract"
   },
   {
     "id": "step_4",
     "type": "send_notification",
     "channel": "email",
     "template": "offer_accepted_buyer",
     "to": "{{offer.contact_id}}",
     "to_type": "contact"
   },
   {
     "id": "step_5",
     "type": "send_notification",
     "channel": "email",
     "template": "offer_accepted_seller",
     "to_role": "owner",
     "related_entity": "property",
     "related_id": "{{offer.property_id}}"
   },
   {
     "id": "step_6",
     "type": "log_activity",
     "entity_type": "property",
     "entity_id": "{{offer.property_id}}",
     "activity_type": "offer_accepted",
     "title": "Offer accepted: {{offer.offer_price}}"
   }
 ]'::jsonb,
 '{"nodes": [], "edges": []}'::jsonb)
ON CONFLICT (id) DO UPDATE SET actions = EXCLUDED.actions;

-- Email templates
INSERT INTO notification_templates (id, tenant_id, name, label, channel, subject, body, is_system, variables)
VALUES
('bf000003-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_accepted_buyer', 'Offer Accepted - Buyer', 'email',
 'Great news! Your offer has been accepted',
 E'Dear {{first_name}},\n\nWe are delighted to inform you that your offer for {{property_title}} has been accepted!\n\nOffer Amount: {{offer_amount}}\n\nOur team will be in touch shortly to proceed with the contract.\n\nBest regards,\nThe Real Estate Team',
 true, '["first_name", "property_title", "offer_amount"]'::jsonb),
('bf000003-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'offer_accepted_seller', 'Offer Accepted - Seller', 'email',
 'Your property has received an accepted offer',
 E'Dear {{first_name}},\n\nGreat news! An offer for your property {{property_title}} has been accepted.\n\nOffer Amount: {{offer_amount}}\nBuyer: {{buyer_name}}\n\nWe will contact you to proceed with the contract process.\n\nBest regards,\nThe Real Estate Team',
 true, '["first_name", "property_title", "offer_amount", "buyer_name"]'::jsonb)
ON CONFLICT (tenant_id, name) DO UPDATE SET subject = EXCLUDED.subject, body = EXCLUDED.body;
