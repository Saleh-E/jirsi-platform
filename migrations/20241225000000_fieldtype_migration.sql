-- Migration Script: Field Type Compatibility
-- Purpose: Ensure field_type values are compatible with frontend
-- NOTE: This is a documentation/reference migration - actual tables are field_defs
-- Created: 2024-12-25

-- ============================================================================
-- FIELD TYPE NORMALIZATION
-- ============================================================================

-- This migration ensures all field_type values in field_defs table are normalized
-- Supported field types: text, email, tel, url, number, integer, currency, date, 
-- datetime, select, multiselect, boolean, textarea, longtext, lookup, file, file_array

-- Normalize any legacy 'Select' types (case-sensitive fix)
UPDATE field_defs 
SET field_type = 'select' 
WHERE field_type = 'Select';

-- Normalize any legacy 'Link' types to 'lookup'
UPDATE field_defs 
SET field_type = 'lookup' 
WHERE field_type IN ('Link', 'link', 'association');

-- Normalize 'Dropdown' to 'select'
UPDATE field_defs 
SET field_type = 'select' 
WHERE field_type = 'Dropdown';

-- Normalize 'money' to 'currency'
UPDATE field_defs 
SET field_type = 'currency' 
WHERE field_type = 'money';

-- Normalize 'richtext' to 'longtext'
UPDATE field_defs 
SET field_type = 'longtext' 
WHERE field_type = 'richtext';

-- Migration complete
