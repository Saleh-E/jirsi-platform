-- ============================================================================
-- PROJECT ANTIGRAVITY: Diamond Layers Migration
-- ============================================================================
-- This migration adds the Diamond Architecture columns to field_defs:
-- - layout: LayoutConfig (visibility, readonly conditions, form layout)
-- - physics: MergeStrategy (CRDT sync strategy)
-- - intelligence: AiMetadata (AI/LLM hints)
-- - rules: ValidationRule[] (portable + async validation)
-- - is_system: boolean (system field flag)
-- ============================================================================

-- Add Antigravity Diamond Layers to field definitions
ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS layout JSONB DEFAULT '{
    "formSpan": 12,
    "section": null,
    "visibleIf": {"op": "always"},
    "readonlyIf": {"op": "never"}
}'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS physics JSONB DEFAULT '"lastWriteWins"'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS intelligence JSONB DEFAULT '{
    "description": null,
    "isPii": false,
    "embed": false,
    "autoGenerate": false
}'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS rules JSONB DEFAULT '[]'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS is_system BOOLEAN DEFAULT false NOT NULL;

-- ============================================================================
-- INDEXES for high-performance lookup
-- ============================================================================

-- System fields are often filtered
CREATE INDEX IF NOT EXISTS idx_field_defs_is_system 
ON field_defs(is_system);

-- Layout section for form grouping
CREATE INDEX IF NOT EXISTS idx_field_defs_layout_section 
ON field_defs((layout->>'section')) 
WHERE layout->>'section' IS NOT NULL;

-- PII fields for compliance filtering
CREATE INDEX IF NOT EXISTS idx_field_defs_intelligence_pii 
ON field_defs((intelligence->>'isPii')) 
WHERE (intelligence->>'isPii')::boolean = true;

-- ============================================================================
-- MIGRATE EXISTING DATA
-- ============================================================================

-- Convert legacy is_readonly to layout.readonlyIf
UPDATE field_defs
SET layout = jsonb_set(
    layout,
    '{readonlyIf}',
    CASE WHEN is_readonly = true 
        THEN '{"op": "always"}'::jsonb 
        ELSE '{"op": "never"}'::jsonb 
    END
)
WHERE layout->>'readonlyIf' IS NULL OR layout->'readonlyIf' = '{"op": "never"}'::jsonb;

-- Convert legacy is_required to rules
UPDATE field_defs
SET rules = rules || '[{"rule": "required"}]'::jsonb
WHERE is_required = true 
AND NOT (rules @> '[{"rule": "required"}]'::jsonb);

-- ============================================================================
-- COMMENTS for documentation
-- ============================================================================

COMMENT ON COLUMN field_defs.layout IS 'Antigravity Layout: form_span, section, visible_if, readonly_if conditions';
COMMENT ON COLUMN field_defs.physics IS 'Antigravity Physics: CRDT merge strategy (lastWriteWins, textMerge, appendOnly)';
COMMENT ON COLUMN field_defs.intelligence IS 'Antigravity Intelligence: AI metadata (is_pii, embed, auto_generate)';
COMMENT ON COLUMN field_defs.rules IS 'Antigravity Rules: Validation rules array (required, minLength, email, unique, etc.)';
COMMENT ON COLUMN field_defs.is_system IS 'System-managed field flag (cannot be deleted by users)';
