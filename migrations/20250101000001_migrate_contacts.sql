-- Migrate contacts table to entity_records

INSERT INTO entity_records (id, tenant_id, entity_type_id, data, created_at, updated_at)
SELECT
    c.id,
    c.tenant_id,
    et.id,
    jsonb_build_object(
        'first_name', c.first_name,
        'last_name', c.last_name,
        'email', c.email,
        'phone', c.phone,
        'company_id', c.company_id,
        'job_title', c.job_title,
        'lifecycle_stage', c.lifecycle_stage,
        'lead_source', c.lead_source,
        'owner_id', c.owner_id,
        'tags', c.tags
    ) || c.custom_fields, -- Merge custom_fields into the main data object
    c.created_at,
    c.updated_at
FROM contacts c
JOIN entity_types et ON c.tenant_id = et.tenant_id AND et.name = 'contact'
ON CONFLICT (id) DO NOTHING;
