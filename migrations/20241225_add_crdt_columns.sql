-- Add CRDT columns for collaborative text fields
-- Migration: 20241225_add_crdt_columns

-- Add CRDT state columns to entity_records
ALTER TABLE entity_records 
ADD COLUMN description_crdt BYTEA,
ADD COLUMN notes_crdt BYTEA;

-- Add CRDT columns to tasks table (if exists)
ALTER TABLE IF EXISTS tasks
ADD COLUMN description_crdt BYTEA,
ADD COLUMN notes_crdt BYTEA;

-- Add CRDT columns to comments table (if exists)
ALTER TABLE IF EXISTS comments
ADD COLUMN content_crdt BYTEA;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_entity_records_crdt 
ON entity_records(id) WHERE description_crdt IS NOT NULL OR notes_crdt IS NOT NULL;

-- Comments for documentation
COMMENT ON COLUMN entity_records.description_crdt IS 'Yrs CRDT state for collaborative description editing';
COMMENT ON COLUMN entity_records.notes_crdt IS 'Yrs CRDT state for collaborative notes editing';
