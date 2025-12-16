-- Fix field_type format: convert VARCHAR strings to JSON format for FieldType enum
-- FieldType enum expects: {"type": "Select"} not "select"

-- Select type
UPDATE field_defs SET field_type = '{"type": "Select", "config": {"options": []}}' WHERE field_type = 'select';

-- Status type (treat as Select)
UPDATE field_defs SET field_type = '{"type": "Select", "config": {"options": []}}' WHERE field_type = 'status';

-- Text type
UPDATE field_defs SET field_type = '{"type": "Text"}' WHERE field_type = 'text';

-- Email type
UPDATE field_defs SET field_type = '{"type": "Email"}' WHERE field_type = 'email';

-- Phone type
UPDATE field_defs SET field_type = '{"type": "Phone"}' WHERE field_type = 'phone';

-- Number type
UPDATE field_defs SET field_type = '{"type": "Number", "config": {"decimals": null}}' WHERE field_type = 'number';

-- Integer type
UPDATE field_defs SET field_type = '{"type": "Number", "config": {"decimals": 0}}' WHERE field_type = 'integer';

-- Money type
UPDATE field_defs SET field_type = '{"type": "Money", "config": {"currency_code": "USD"}}' WHERE field_type = 'money';

-- Currency type
UPDATE field_defs SET field_type = '{"type": "Money", "config": {"currency_code": "USD"}}' WHERE field_type = 'currency';

-- Date type
UPDATE field_defs SET field_type = '{"type": "Date"}' WHERE field_type = 'date';

-- DateTime type
UPDATE field_defs SET field_type = '{"type": "DateTime"}' WHERE field_type = 'datetime';

-- Boolean type
UPDATE field_defs SET field_type = '{"type": "Boolean"}' WHERE field_type = 'boolean';

-- TextArea/longtext type
UPDATE field_defs SET field_type = '{"type": "TextArea"}' WHERE field_type = 'textarea' OR field_type = 'longtext';

-- URL type
UPDATE field_defs SET field_type = '{"type": "Url"}' WHERE field_type = 'url';

-- Link type
UPDATE field_defs SET field_type = '{"type": "Link", "config": {"target_entity": "contact"}}' WHERE field_type = 'link' OR field_type = 'lookup';

-- TagList type
UPDATE field_defs SET field_type = '{"type": "TagList"}' WHERE field_type = 'tag_list';

-- MultiSelect type  
UPDATE field_defs SET field_type = '{"type": "MultiSelect", "config": {"options": []}}' WHERE field_type = 'multiselect';

-- Score type
UPDATE field_defs SET field_type = '{"type": "Score", "config": {"max_value": 100}}' WHERE field_type = 'score';

-- Attachment type
UPDATE field_defs SET field_type = '{"type": "Attachment"}' WHERE field_type = 'attachment' OR field_type = 'file_array';
