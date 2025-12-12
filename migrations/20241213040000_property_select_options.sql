-- Phase 1B Task P1B-04: Property Select Options
-- Adds options to select fields for Property entity

-- Property Type Options
UPDATE field_defs 
SET options = '[
    {"value": "apartment", "label": "Apartment"},
    {"value": "villa", "label": "Villa"},
    {"value": "townhouse", "label": "Townhouse"},
    {"value": "penthouse", "label": "Penthouse"},
    {"value": "land", "label": "Land"},
    {"value": "commercial", "label": "Commercial"},
    {"value": "office", "label": "Office"}
]'::jsonb
WHERE id = 'f1000001-0000-0000-0000-000000000003';

-- Usage Options (Sale/Rent)
UPDATE field_defs 
SET options = '[
    {"value": "sale", "label": "For Sale"},
    {"value": "rent", "label": "For Rent"},
    {"value": "both", "label": "Sale or Rent"}
]'::jsonb
WHERE id = 'f1000001-0000-0000-0000-000000000004';

-- Status Options (Pipeline Stages)
UPDATE field_defs 
SET options = '[
    {"value": "draft", "label": "Draft"},
    {"value": "active", "label": "Active"},
    {"value": "reserved", "label": "Reserved"},
    {"value": "under_offer", "label": "Under Offer"},
    {"value": "sold", "label": "Sold"},
    {"value": "rented", "label": "Rented"},
    {"value": "withdrawn", "label": "Withdrawn"}
]'::jsonb
WHERE id = 'f1000001-0000-0000-0000-000000000005';

-- Currency Options
UPDATE field_defs 
SET options = '[
    {"value": "USD", "label": "USD ($)"},
    {"value": "AED", "label": "AED (د.إ)"},
    {"value": "EUR", "label": "EUR (€)"},
    {"value": "GBP", "label": "GBP (£)"},
    {"value": "SAR", "label": "SAR (ر.س)"}
]'::jsonb
WHERE id = 'f1000001-0000-0000-0000-000000000020';

-- Amenities Options (Multiselect)
UPDATE field_defs 
SET options = '[
    {"value": "pool", "label": "Swimming Pool"},
    {"value": "gym", "label": "Gym"},
    {"value": "parking", "label": "Parking"},
    {"value": "security", "label": "24/7 Security"},
    {"value": "garden", "label": "Garden"},
    {"value": "balcony", "label": "Balcony"},
    {"value": "maid_room", "label": "Maid Room"},
    {"value": "storage", "label": "Storage"},
    {"value": "concierge", "label": "Concierge"},
    {"value": "beach_access", "label": "Beach Access"},
    {"value": "children_play", "label": "Children Play Area"},
    {"value": "bbq", "label": "BBQ Area"},
    {"value": "jacuzzi", "label": "Jacuzzi"},
    {"value": "sauna", "label": "Sauna"},
    {"value": "tennis", "label": "Tennis Court"}
]'::jsonb
WHERE id = 'f1000001-0000-0000-0000-000000000027';
