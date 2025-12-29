-- ============================================================================
-- PROJECT ANTIGRAVITY: Diamond Layers Migration (Phase 2)
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
ADD COLUMN IF NOT EXISTS layout JSONB DEFAULT '{}'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS physics JSONB DEFAULT '"lastWriteWins"'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS intelligence JSONB DEFAULT '{}'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS rules JSONB DEFAULT '[]'::jsonb NOT NULL;

ALTER TABLE field_defs
ADD COLUMN IF NOT EXISTS is_system BOOLEAN DEFAULT false NOT NULL;

-- Index for fast system field lookups
CREATE INDEX IF NOT EXISTS idx_field_defs_is_system ON field_defs(is_system);

-- ============================================================================
-- COMMENTS for documentation
-- ============================================================================

COMMENT ON COLUMN field_defs.layout IS 'Antigravity Layout: form_span, section, visible_if, readonly_if conditions';
COMMENT ON COLUMN field_defs.physics IS 'Antigravity Physics: CRDT merge strategy (lastWriteWins, textMerge, appendOnly)';
COMMENT ON COLUMN field_defs.intelligence IS 'Antigravity Intelligence: AI metadata (is_pii, embed, auto_generate)';
COMMENT ON COLUMN field_defs.rules IS 'Antigravity Rules: Validation rules array (required, minLength, email, unique, etc.)';
COMMENT ON COLUMN field_defs.is_system IS 'System-managed field flag (cannot be deleted by users)';
