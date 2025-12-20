-- Phase 3 Task P3-11: Generic UI Filters & Sorting
-- Add filter and sort support to view_defs

-- ============================================================================
-- ADD FILTER CONFIGURATION TO ENTITIES
-- ============================================================================

-- Update view_defs to include filter configurations
-- Each entity gets default filterable fields

-- Property filters
UPDATE view_defs 
SET filters = '[
  {"field": "status", "type": "multi_select", "label": "Status"},
  {"field": "property_type", "type": "multi_select", "label": "Type"},
  {"field": "listing_type", "type": "multi_select", "label": "Listing Type"},
  {"field": "price", "type": "range", "label": "Price"},
  {"field": "bedrooms", "type": "range", "label": "Bedrooms"}
]'::jsonb,
sort = '[{"field": "created_at", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000010' AND view_type = 'table';

-- Listing filters
UPDATE view_defs 
SET filters = '[
  {"field": "listing_status", "type": "multi_select", "label": "Status"},
  {"field": "channel", "type": "multi_select", "label": "Channel"},
  {"field": "list_price", "type": "range", "label": "Price"},
  {"field": "go_live_date", "type": "date_range", "label": "Go Live Date"}
]'::jsonb,
sort = '[{"field": "created_at", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000020' AND view_type = 'table';

-- Viewing filters
UPDATE view_defs 
SET filters = '[
  {"field": "status", "type": "multi_select", "label": "Status"},
  {"field": "scheduled_start", "type": "date_range", "label": "Scheduled Date"},
  {"field": "agent_id", "type": "lookup", "label": "Agent"}
]'::jsonb,
sort = '[{"field": "scheduled_start", "direction": "asc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000011' AND view_type = 'table';

-- Offer filters
UPDATE view_defs 
SET filters = '[
  {"field": "status", "type": "multi_select", "label": "Status"},
  {"field": "finance_type", "type": "multi_select", "label": "Finance Type"},
  {"field": "offer_price", "type": "range", "label": "Offer Amount"},
  {"field": "currency", "type": "multi_select", "label": "Currency"}
]'::jsonb,
sort = '[{"field": "created_at", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000012' AND view_type = 'table';

-- Contract filters
UPDATE view_defs 
SET filters = '[
  {"field": "status", "type": "multi_select", "label": "Status"},
  {"field": "contract_type", "type": "multi_select", "label": "Type"},
  {"field": "start_date", "type": "date_range", "label": "Start Date"},
  {"field": "contract_value", "type": "range", "label": "Value"}
]'::jsonb,
sort = '[{"field": "start_date", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000021' AND view_type = 'table';

-- Contact filters
UPDATE view_defs 
SET filters = '[
  {"field": "lifecycle_stage", "type": "multi_select", "label": "Stage"},
  {"field": "lead_status", "type": "multi_select", "label": "Lead Status"},
  {"field": "contact_type", "type": "multi_select", "label": "Type"}
]'::jsonb,
sort = '[{"field": "created_at", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000001' AND view_type = 'table';

-- Deal filters
UPDATE view_defs 
SET filters = '[
  {"field": "stage", "type": "multi_select", "label": "Stage"},
  {"field": "amount", "type": "range", "label": "Amount"},
  {"field": "expected_close_date", "type": "date_range", "label": "Close Date"}
]'::jsonb,
sort = '[{"field": "created_at", "direction": "desc"}]'::jsonb
WHERE entity_type_id = 'e0000000-0000-0000-0000-000000000003' AND view_type = 'table';
