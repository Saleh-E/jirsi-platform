-- World-Class UX Fix: Populate all select field options
-- Run this to fix "No options found" in all dropdowns

-- ============ PROPERTY ENTITY ============

-- Property Type
UPDATE field_defs SET options = '["apartment","villa","townhouse","penthouse","land","commercial","office","studio","duplex","retail"]'
WHERE name = 'property_type' AND options IS NULL;

-- Status (for properties)
UPDATE field_defs SET options = '["active","draft","sold","rented","reserved","off_market","under_offer","pending"]'
WHERE name = 'status' AND options IS NULL;

-- Usage
UPDATE field_defs SET options = '["sale","rent","both"]'
WHERE name = 'usage' AND options IS NULL;

-- Furnishing
UPDATE field_defs SET options = '["furnished","unfurnished","semi_furnished"]'
WHERE name = 'furnishing' AND options IS NULL;

-- ============ LISTING ENTITY ============

-- Channel
UPDATE field_defs SET options = '["bayut","property_finder","dubizzle","direct","website","social_media","referral"]'
WHERE name = 'channel' AND options IS NULL;

-- Listing Status
UPDATE field_defs SET options = '["active","draft","published","expired","paused"]'
WHERE name = 'listing_status' AND options IS NULL;

-- ============ CONTACT ENTITY ============

-- Lifecycle Stage
UPDATE field_defs SET options = '["subscriber","lead","mql","sql","opportunity","customer","evangelist"]'
WHERE name = 'lifecycle_stage' AND options IS NULL;

-- Lead Source
UPDATE field_defs SET options = '["website","referral","social_media","cold_call","event","partner","advertisement"]'
WHERE name = 'lead_source' AND options IS NULL;

-- Contact Type
UPDATE field_defs SET options = '["buyer","seller","tenant","landlord","investor","agent"]'
WHERE name = 'contact_type' AND options IS NULL;

-- ============ DEAL ENTITY ============

-- Deal Stage/Stage
UPDATE field_defs SET options = '["new","contacted","qualified","proposal","negotiation","won","lost"]'
WHERE name IN ('stage', 'deal_stage') AND options IS NULL;

-- Deal Priority
UPDATE field_defs SET options = '["low","medium","high","urgent"]'
WHERE name = 'priority' AND options IS NULL;

-- ============ TASK ENTITY ============

-- Task Status
UPDATE field_defs SET options = '["open","in_progress","completed","cancelled","deferred"]'
WHERE name = 'task_status' AND options IS NULL;

-- Task Type  
UPDATE field_defs SET options = '["call","email","meeting","follow_up","viewing","site_visit","documentation","other"]'
WHERE name = 'task_type' AND options IS NULL;

-- Task Priority
UPDATE field_defs SET options = '["low","medium","high","urgent"]'
WHERE name = 'priority' AND options IS NULL;

-- ============ VIEWING ENTITY ============

-- Viewing Status
UPDATE field_defs SET options = '["scheduled","confirmed","completed","cancelled","no_show","rescheduled"]'
WHERE name = 'viewing_status' AND options IS NULL;

-- Viewing Type
UPDATE field_defs SET options = '["in_person","virtual","video_call"]'
WHERE name = 'viewing_type' AND options IS NULL;

-- ============ OFFER ENTITY ============

-- Offer Status
UPDATE field_defs SET options = '["pending","accepted","rejected","countered","expired","withdrawn"]'
WHERE name = 'offer_status' AND options IS NULL;

-- ============ COMMON FIELDS ============

-- Country
UPDATE field_defs SET options = '["UAE","Saudi Arabia","Qatar","Bahrain","Kuwait","Oman","Egypt","Jordan"]'
WHERE name = 'country' AND options IS NULL;

-- City (UAE focused)
UPDATE field_defs SET options = '["Dubai","Abu Dhabi","Sharjah","Ajman","Ras Al Khaimah","Fujairah","Umm Al Quwain"]'
WHERE name = 'city' AND options IS NULL;

-- Currency
UPDATE field_defs SET options = '["AED","USD","EUR","GBP","SAR","QAR"]'
WHERE name = 'currency' AND options IS NULL;

-- Also update any status fields that might be using different naming
UPDATE field_defs SET options = '["active","inactive","pending","archived"]'
WHERE name LIKE '%status%' AND field_type LIKE '%Select%' AND (options IS NULL OR options = '[]');

-- Verify updates
SELECT name, entity_type_id, options FROM field_defs WHERE options IS NOT NULL AND options != '[]' ORDER BY name;
