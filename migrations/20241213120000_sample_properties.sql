-- Phase 1B Task P1B-12: Sample Properties Seed
-- Creates 10 sample properties for testing with various types and statuses

INSERT INTO properties (id, tenant_id, reference, title, property_type, usage, status, 
    city, area, bedrooms, bathrooms, size_sqm, price, rent_amount, currency, description,
    created_at, updated_at)
VALUES
-- Active properties (for Kanban active column)
('a0000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-001', 'Luxury Marina Villa', 'villa', 'sale', 'active', 
 'Dubai', 'Marina', 5, 6, 500, 5500000, NULL, 'AED',
 'Stunning 5BR villa with private pool and marina views. Modern design with high-end finishes.',
 NOW(), NOW()),

('a0000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-002', 'Downtown Studio', 'apartment', 'rent', 'active', 
 'Dubai', 'Downtown', 0, 1, 45, NULL, 80000, 'AED',
 'Cozy studio apartment with stunning Burj Khalifa views. Fully furnished and ready to move in.',
 NOW(), NOW()),

('a0000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-003', 'JBR 2BR Apartment', 'apartment', 'both', 'active', 
 'Dubai', 'JBR', 2, 2, 120, 1800000, 120000, 'AED',
 'Sea-facing apartment in JBR. Walking distance to the beach. Perfect for families.',
 NOW(), NOW()),

-- Reserved property
('a0000001-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-004', 'Palm Penthouse', 'penthouse', 'sale', 'reserved', 
 'Dubai', 'Palm Jumeirah', 4, 5, 400, 12000000, NULL, 'AED',
 'Exclusive penthouse on the Palm with 360-degree views. Private elevator and rooftop terrace.',
 NOW(), NOW()),

-- Under Offer
('a0000001-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-005', 'Arabian Ranches Villa', 'villa', 'sale', 'under_offer', 
 'Dubai', 'Arabian Ranches', 4, 4, 350, 3500000, NULL, 'AED',
 'Family villa in gated community. Large garden, private pool, and community facilities.',
 NOW(), NOW()),

-- Sold
('a0000001-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-006', 'Springs Townhouse', 'townhouse', 'sale', 'sold', 
 'Dubai', 'Springs', 3, 3, 220, 2200000, NULL, 'AED',
 'Recently renovated townhouse with landscaped garden. Community pool and parks.',
 NOW(), NOW()),

-- Rented
('a0000001-0000-0000-0000-000000000007', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-007', 'Motor City Apartment', 'apartment', 'rent', 'rented', 
 'Dubai', 'Motor City', 1, 1, 75, NULL, 45000, 'AED',
 'Modern 1BR apartment close to Dubai Autodrome. Ideal for motorsport enthusiasts.',
 NOW(), NOW()),

-- Draft
('a0000001-0000-0000-0000-000000000008', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-008', 'DIFC Commercial', 'commercial', 'rent', 'draft', 
 'Dubai', 'DIFC', 0, 4, 500, NULL, 500000, 'AED',
 'Prime commercial space in DIFC. Perfect for financial services or luxury retail.',
 NOW(), NOW()),

-- More active for variety
('a0000001-0000-0000-0000-000000000009', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-009', 'Jumeirah Beach Villa', 'villa', 'sale', 'active', 
 'Dubai', 'Jumeirah', 6, 7, 800, 15000000, NULL, 'AED',
 'Beachfront villa with private beach access. Mediterranean style with modern amenities.',
 NOW(), NOW()),

('a0000001-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'PROP-010', 'Business Bay Office', 'office', 'rent', 'active', 
 'Dubai', 'Business Bay', 0, 2, 200, NULL, 150000, 'AED',
 'Modern office space with panoramic canal views. Ready for immediate occupation.',
 NOW(), NOW())

ON CONFLICT (id) DO NOTHING;

