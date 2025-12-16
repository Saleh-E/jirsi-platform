-- Add options to existing select/status fields that are missing them

-- Contact: lifecycle_stage
UPDATE field_defs 
SET options = '["Subscriber", "Lead", "MQL", "SQL", "Opportunity", "Customer", "Evangelist"]'::jsonb
WHERE name = 'lifecycle_stage' AND field_type = 'select' AND options IS NULL;

-- Contact: lead_score - add as taglist/select with common values
UPDATE field_defs 
SET options = '["Cold", "Warm", "Hot", "Very Hot"]'::jsonb
WHERE name = 'lead_score' AND options IS NULL;

-- Contact & Company: tags - add empty array for taglist
UPDATE field_defs 
SET options = '[]'::jsonb
WHERE name = 'tags' AND options IS NULL;

-- Company: industry
UPDATE field_defs 
SET options = '["Technology", "Finance", "Healthcare", "Retail", "Real Estate", "Other"]'::jsonb
WHERE name = 'industry' AND field_type = 'select' AND options IS NULL;

-- Company: size
UPDATE field_defs 
SET options = '["1-10", "11-50", "51-200", "201-500", "500+"]'::jsonb
WHERE name = 'size' AND field_type = 'select' AND options IS NULL;

-- Deal: stage
UPDATE field_defs 
SET options = '["New", "Contacted", "Qualified", "Proposal", "Negotiation", "Won", "Lost"]'::jsonb
WHERE name = 'stage' AND field_type = 'select' AND options IS NULL;

-- Property: status
UPDATE field_defs 
SET options = '["Available", "Under Offer", "Sold", "Rented", "Off Market"]'::jsonb
WHERE name = 'status' AND field_type = 'select' AND options IS NULL;

-- Property: property_type
UPDATE field_defs 
SET options = '["Apartment", "Villa", "Townhouse", "Penthouse", "Land", "Commercial"]'::jsonb
WHERE name = 'property_type' AND field_type = 'select' AND options IS NULL;

-- Task: priority
UPDATE field_defs 
SET options = '["Low", "Medium", "High", "Urgent"]'::jsonb
WHERE name = 'priority' AND field_type = 'select' AND options IS NULL;

-- Task: status
UPDATE field_defs 
SET options = '["Open", "In Progress", "Completed", "Cancelled"]'::jsonb
WHERE name = 'status' AND field_type = 'select' AND options IS NULL;

-- Task: task_type
UPDATE field_defs 
SET options = '["Call", "Email", "Meeting", "Follow Up", "Other"]'::jsonb
WHERE name = 'task_type' AND field_type = 'select' AND options IS NULL;
