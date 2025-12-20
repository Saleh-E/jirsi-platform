-- Add deleted_at to entity_records for soft delete support
ALTER TABLE entity_records ADD COLUMN deleted_at TIMESTAMPTZ;
CREATE INDEX idx_entity_records_deleted_at ON entity_records(deleted_at);
