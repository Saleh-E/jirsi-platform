-- Phase 3 Task P3-07: Kanban Views by Status
-- Configure Kanban views for all Real Estate entities

-- ============================================================================
-- PROPERTY KANBAN VIEW
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
('b8000007-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000010',
 'properties_kanban', 'Pipeline', 'kanban', false, true,
 '[]'::jsonb, '[]'::jsonb, '[]'::jsonb,
 '{
   "group_by_field": "status",
   "card_title_field": "title",
   "card_subtitle_field": "location",
   "card_fields": ["price", "bedrooms", "bathrooms", "property_type"],
   "column_order": ["draft", "active", "reserved", "under_offer", "sold", "rented", "withdrawn"],
   "column_colors": {
     "draft": "#6b7280",
     "active": "#22c55e",
     "reserved": "#f59e0b",
     "under_offer": "#8b5cf6",
     "sold": "#10b981",
     "rented": "#06b6d4",
     "withdrawn": "#ef4444"
   }
 }'::jsonb)
ON CONFLICT (id) DO UPDATE SET settings = EXCLUDED.settings;

-- ============================================================================
-- LISTING KANBAN VIEW (already created, update)
-- ============================================================================

UPDATE view_defs 
SET settings = '{
  "group_by_field": "listing_status",
  "card_title_field": "headline",
  "card_subtitle_field": "property_id",
  "card_fields": ["list_price", "channel", "go_live_date"],
  "column_order": ["draft", "live", "paused", "expired"],
  "column_colors": {
    "draft": "#6b7280",
    "live": "#22c55e",
    "paused": "#f59e0b",
    "expired": "#ef4444"
  }
}'::jsonb
WHERE name = 'listings_pipeline';

-- ============================================================================
-- VIEWING KANBAN VIEW (update existing)
-- ============================================================================

UPDATE view_defs 
SET settings = '{
  "group_by_field": "status",
  "card_title_field": "property_id",
  "card_subtitle_field": "contact_id",
  "card_fields": ["scheduled_start", "agent_id"],
  "column_order": ["scheduled", "completed", "noshow", "cancelled"],
  "column_colors": {
    "scheduled": "#3b82f6",
    "completed": "#22c55e",
    "noshow": "#f59e0b",
    "cancelled": "#ef4444"
  }
}'::jsonb
WHERE name = 'viewings_pipeline';

-- ============================================================================
-- OFFER KANBAN VIEW (update existing)
-- ============================================================================

UPDATE view_defs 
SET settings = '{
  "group_by_field": "status",
  "card_title_field": "offer_price",
  "card_subtitle_field": "contact_id",
  "card_fields": ["property_id", "finance_type", "currency"],
  "column_order": ["new", "countered", "accepted", "rejected", "withdrawn"],
  "column_colors": {
    "new": "#3b82f6",
    "countered": "#f59e0b",
    "accepted": "#22c55e",
    "rejected": "#ef4444",
    "withdrawn": "#6b7280"
  }
}'::jsonb
WHERE name = 'offers_pipeline';

-- ============================================================================
-- CONTRACT KANBAN VIEW (update existing)
-- ============================================================================

UPDATE view_defs 
SET settings = '{
  "group_by_field": "status",
  "card_title_field": "contract_value",
  "card_subtitle_field": "property_id",
  "card_fields": ["contract_type", "start_date", "buyer_contact_id"],
  "column_order": ["draft", "active", "completed", "terminated"],
  "column_colors": {
    "draft": "#6b7280",
    "active": "#22c55e",
    "completed": "#3b82f6",
    "terminated": "#ef4444"
  }
}'::jsonb
WHERE name = 'contracts_pipeline';
