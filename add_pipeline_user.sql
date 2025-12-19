-- Add pipeline and user

INSERT INTO pipelines (id, tenant_id, name, is_default, stages)
VALUES (
    gen_random_uuid(),
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'Sales Pipeline',
    true,
    '[{"id": "lead", "name": "Lead"}, {"id": "qualified", "name": "Qualified"}, {"id": "proposal", "name": "Proposal"}, {"id": "negotiation", "name": "Negotiation"}, {"id": "closed_won", "name": "Closed Won"}, {"id": "closed_lost", "name": "Closed Lost"}]'::jsonb
) ON CONFLICT DO NOTHING;

INSERT INTO users (id, tenant_id, email, name, password_hash, role, status)
VALUES (
    gen_random_uuid(),
    'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
    'admin@demo.com',
    'Admin User',
    '$argon2id$v=19$m=19456,t=2,p=1$dummy',
    'admin',
    'active'
) ON CONFLICT DO NOTHING;

SELECT 'Pipeline created' as status, COUNT(*) as count FROM pipelines WHERE tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e'
UNION ALL
SELECT 'User created', COUNT(*) FROM users WHERE tenant_id = 'b128c8da-6e56-485d-b2fe-e45fb7492b2e';
