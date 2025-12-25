-- Migration Script: Convert Legacy Field Types to New Schema
-- Purpose: Migrate Select → Dropdown and Link → Association
-- WARNING: Always backup your database before running this migration!
-- Created: 2025-12-24

-- ============================================================================
-- PART 1: Backup Tables (Safety First!)
-- ============================================================================

-- Create backup of field definitions
CREATE TABLE IF NOT EXISTS field_definitions_backup_20251224 AS 
SELECT * FROM field_definitions;

-- Create backup of entity metadata if it exists
CREATE TABLE IF NOT EXISTS entity_metadata_backup_20251224 AS 
SELECT * FROM entity_metadata;

-- ============================================================================
-- PART 2: Migrate Select → Dropdown
-- ============================================================================

-- Update field_type from 'Select' to 'Dropdown'
-- This assumes the field_type is stored as a string/enum
UPDATE field_definitions
SET 
    field_type = 'Dropdown',
    metadata = jsonb_set(
        COALESCE(metadata, '{}'::jsonb),
        '{allow_create}',
        'false'::jsonb
    )
WHERE field_type = 'Select'
  AND (metadata IS NULL OR metadata->>'migrated' IS NULL);

-- If using PostgreSQL JSONB for storing field configuration:
-- Convert options array to SelectChoice format
UPDATE field_definitions
SET metadata = jsonb_set(
    metadata,
    '{options}',
    (
        SELECT jsonb_agg(
            jsonb_build_object(
                'value', opt,
                'label', opt,
                'color', null,
                'icon', null,
                'is_default', false,
                'sort_order', idx
            )
        )
        FROM jsonb_array_elements_text(metadata->'options') WITH ORDINALITY AS t(opt, idx)
    )
)
WHERE field_type = 'Dropdown'
  AND metadata->'options' IS NOT NULL
  AND jsonb_typeof(metadata->'options') = 'array'
  AND (metadata->>'migrated' IS NULL);

-- Mark as migrated
UPDATE field_definitions
SET metadata = jsonb_set(
    COALESCE(metadata, '{}'::jsonb),
    '{migrated}',
    'true'::jsonb
)
WHERE field_type = 'Dropdown'
  AND (metadata->>'migrated' IS NULL);

-- ============================================================================
-- PART 3: Migrate Link → Association
-- ============================================================================

-- Update field_type from 'Link' to 'Association'
UPDATE field_definitions
SET 
    field_type = 'Association',
    metadata = jsonb_set(
        jsonb_set(
            COALESCE(metadata, '{}'::jsonb),
            '{display_field}',
            '"name"'::json b
        ),
        '{allow_inline_create}',
        'false'::jsonb
    )
WHERE field_type = 'Link'
  AND (metadata->>'migrated' IS NULL);

-- Ensure target_entity is set (if stored in metadata)
UPDATE field_definitions
SET metadata = jsonb_set(
    metadata,
    '{target_entity}',
    to_jsonb(metadata->>'target_entity')
)
WHERE field_type = 'Association'
  AND metadata->>'target_entity' IS NOT NULL
  AND (metadata->>'migrated' IS NULL);

-- Mark as migrated
UPDATE field_definitions
SET metadata = jsonb_set(
    COALESCE(metadata, '{}'::jsonb),
    '{migrated}',
    'true'::jsonb
)
WHERE field_type = 'Association'
  AND (metadata->>'migrated' IS NULL);

-- ============================================================================
-- PART 4: Data Validation & Reporting
-- ============================================================================

-- Count migrated fields
DO $$
DECLARE
    dropdown_count INTEGER;
    association_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO dropdown_count FROM field_definitions WHERE field_type = 'Dropdown';
    SELECT COUNT(*) INTO association_count FROM field_definitions WHERE field_type = 'Association';
    
    RAISE NOTICE 'Migration Complete:';
    RAISE NOTICE '  - Dropdown fields: %', dropdown_count;
    RAISE NOTICE '  - Association fields: %', association_count;
END $$;

-- Verify no legacy field types remain
SELECT 
    'WARNING: Legacy field types still exist' AS status,
    field_type,
    COUNT(*) as count
FROM field_definitions
WHERE field_type IN ('Select', 'Link')
GROUP BY field_type;

-- ============================================================================
-- PART 5: Fallback Handler (Application Level)
-- ============================================================================

-- Note: The following is Rust code to be added to the application
-- Add this to your field loading logic:

/*
// In your Rust code (e.g., field.rs or repository.rs)

impl FieldDef {
    /// Migrate legacy field types on-the-fly
    pub fn migrate_if_needed(mut self) -> Self {
        match &self.field_type {
            FieldType::Select { options } => {
                // Convert to Dropdown
                let choices: Vec<SelectChoice> = options.iter().enumerate().map(|(idx, opt)| {
                    SelectChoice {
                        value: opt.clone(),
                        label: opt.clone(),
                        color: None,
                        icon: None,
                        is_default: false,
                        sort_order: idx as i32,
                    }
                }).collect();
                
                self.field_type = FieldType::Dropdown {
                    options: choices,
                    allow_create: false,
                };
            },
            FieldType::Link { target_entity } => {
                // Convert to Association
                self.field_type = FieldType::Association {
                    target_entity: target_entity.clone(),
                    display_field: "name".to_string(),
                    allow_inline_create: false,
                };
            },
            _ => {}
        }
        self
    }
}

// Usage in your repository:
pub async fn get_field_definition(id: Uuid) -> Result<FieldDef> {
    let mut field = query_field_from_db(id).await?;
    field = field.migrate_if_needed(); // Auto-migrate on load
    Ok(field)
}
*/

-- ============================================================================
-- PART 6: Rollback Script (In Case of Issues)
-- ============================================================================

-- Uncomment and run if you need to rollback:
/*
-- Restore from backup
TRUNCATE TABLE field_definitions;
INSERT INTO field_definitions SELECT * FROM field_definitions_backup_20251224;

TRUNCATE TABLE entity_metadata;
INSERT INTO entity_metadata SELECT * FROM entity_metadata_backup_20251224;

-- Drop backup tables
DROP TABLE IF EXISTS field_definitions_backup_20251224;
DROP TABLE IF EXISTS entity_metadata_backup_20251224;
*/

-- ============================================================================
-- COMPLETION
-- ============================================================================

-- After verifying migration success, you can drop the backup tables:
-- DROP TABLE IF EXISTS field_definitions_backup_20251224;
-- DROP TABLE IF EXISTS entity_metadata_backup_20251224;

COMMIT;
