-- Phase 2 Task P2-C03: Notification System & P2-D03: Future Module Templates
-- Defines notification channels and future module metadata patterns

-- ============================================================================
-- NOTIFICATION CHANNELS TABLE
-- Stores configuration for different notification channels
-- ============================================================================

CREATE TABLE IF NOT EXISTS notification_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Channel info
    channel_type VARCHAR(50) NOT NULL, -- 'email', 'sms', 'whatsapp', 'push', 'webhook'
    name VARCHAR(100) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    
    -- Configuration (encrypted in production)
    config JSONB DEFAULT '{}', -- API keys, endpoints, etc
    
    -- Limits
    daily_limit INTEGER,
    monthly_limit INTEGER,
    
    -- Stats
    sent_today INTEGER DEFAULT 0,
    sent_month INTEGER DEFAULT 0,
    last_sent_at TIMESTAMPTZ,
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, channel_type, name)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_notification_channels_tenant ON notification_channels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notification_channels_type ON notification_channels(channel_type);

-- ============================================================================
-- NOTIFICATION LOG TABLE
-- Tracks all sent notifications
-- ============================================================================

CREATE TABLE IF NOT EXISTS notification_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    
    -- Channel and template
    channel_id UUID REFERENCES notification_channels(id),
    template_id UUID REFERENCES notification_templates(id),
    channel_type VARCHAR(50) NOT NULL,
    
    -- Recipients
    recipient_type VARCHAR(50), -- 'contact', 'user', 'email'
    recipient_id UUID,
    recipient_address VARCHAR(255) NOT NULL, -- email, phone, etc
    
    -- Content
    subject VARCHAR(500),
    body TEXT NOT NULL,
    
    -- Status
    status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'sent', 'delivered', 'failed', 'bounced'
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    error_message TEXT,
    
    -- Context
    related_entity_type VARCHAR(100),
    related_entity_id UUID,
    workflow_execution_id UUID REFERENCES workflow_executions(id),
    
    -- System
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_notification_log_tenant ON notification_log(tenant_id);
CREATE INDEX IF NOT EXISTS idx_notification_log_recipient ON notification_log(recipient_address);
CREATE INDEX IF NOT EXISTS idx_notification_log_status ON notification_log(status);
CREATE INDEX IF NOT EXISTS idx_notification_log_created ON notification_log(created_at DESC);

-- ============================================================================
-- SEED DEFAULT NOTIFICATION CHANNELS
-- ============================================================================

INSERT INTO notification_channels (id, tenant_id, channel_type, name, is_active, is_default, config)
VALUES
('ad000001-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'email', 'Primary Email', true, true, 
 '{"provider": "smtp", "from": "noreply@realestate.com", "from_name": "Real Estate Platform"}'::jsonb),
('ad000001-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'sms', 'SMS Gateway', true, false,
 '{"provider": "twilio", "from_number": "+1234567890"}'::jsonb),
('ad000001-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'whatsapp', 'WhatsApp Business', true, false,
 '{"provider": "twilio", "from_number": "+1234567890"}'::jsonb)
ON CONFLICT (tenant_id, channel_type, name) DO NOTHING;

-- ============================================================================
-- FUTURE MODULE TEMPLATES (Appointment & Clinic)
-- ============================================================================

-- App definitions for future modules
INSERT INTO entity_types (id, tenant_id, app_id, name, label, label_plural, icon, description)
VALUES
-- Appointment Module
('e0000000-0000-0000-0000-000000000030', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'appointment', 'appointment', 'Appointment', 'Appointments', 'calendar-check',
 'Scheduled appointments for services'),
('e0000000-0000-0000-0000-000000000031', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'appointment', 'service', 'Service', 'Services', 'briefcase',
 'Services that can be booked'),
('e0000000-0000-0000-0000-000000000032', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'appointment', 'availability', 'Availability', 'Availabilities', 'clock',
 'Time slots available for booking'),

-- Clinic Module  
('e0000000-0000-0000-0000-000000000040', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'clinic', 'patient', 'Patient', 'Patients', 'user-md',
 'Patient records'),
('e0000000-0000-0000-0000-000000000041', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'clinic', 'consultation', 'Consultation', 'Consultations', 'stethoscope',
 'Medical consultations'),
('e0000000-0000-0000-0000-000000000042', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e',
 'clinic', 'prescription', 'Prescription', 'Prescriptions', 'file-medical',
 'Medical prescriptions')
ON CONFLICT (tenant_id, name) DO NOTHING;

-- ============================================================================
-- APPOINTMENT MODULE FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, sort_order, "group")
VALUES
-- Appointment fields
('ad000002-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'contact_id', 'Client', 'lookup', true, true, true, 1, 'Basic'),
('ad000002-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'service_id', 'Service', 'lookup', true, true, true, 2, 'Basic'),
('ad000002-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'scheduled_at', 'Scheduled Time', 'datetime', true, true, true, 3, 'Scheduling'),
('ad000002-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'duration_minutes', 'Duration (min)', 'integer', true, true, false, 4, 'Scheduling'),
('ad000002-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'status', 'Status', 'select', true, true, true, 5, 'Status'),
('ad000002-0000-0000-0000-000000000006', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030', 'notes', 'Notes', 'textarea', false, false, false, 6, 'Details'),

-- Service fields
('ad000002-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000031', 'name', 'Service Name', 'text', true, true, true, 1, 'Basic'),
('ad000002-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000031', 'description', 'Description', 'textarea', false, false, false, 2, 'Basic'),
('ad000002-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000031', 'duration_minutes', 'Duration (min)', 'integer', true, true, false, 3, 'Basic'),
('ad000002-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000031', 'price', 'Price', 'currency', false, true, true, 4, 'Pricing')
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- CLINIC MODULE FIELD DEFINITIONS
-- ============================================================================

INSERT INTO field_defs (id, tenant_id, entity_type_id, name, label, field_type, is_required, show_in_list, show_in_card, sort_order, "group")
VALUES
-- Patient fields (extends contact)
('ad000003-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040', 'contact_id', 'Contact', 'lookup', true, true, true, 1, 'Basic'),
('ad000003-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040', 'date_of_birth', 'Date of Birth', 'date', false, true, false, 2, 'Personal'),
('ad000003-0000-0000-0000-000000000003', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040', 'blood_type', 'Blood Type', 'select', false, true, false, 3, 'Medical'),
('ad000003-0000-0000-0000-000000000004', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040', 'allergies', 'Allergies', 'textarea', false, false, false, 4, 'Medical'),
('ad000003-0000-0000-0000-000000000005', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040', 'medical_history', 'Medical History', 'longtext', false, false, false, 5, 'Medical'),

-- Consultation fields
('ad000003-0000-0000-0000-000000000010', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'patient_id', 'Patient', 'lookup', true, true, true, 1, 'Basic'),
('ad000003-0000-0000-0000-000000000011', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'doctor_id', 'Doctor', 'lookup', true, true, true, 2, 'Basic'),
('ad000003-0000-0000-0000-000000000012', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'date', 'Consultation Date', 'datetime', true, true, true, 3, 'Basic'),
('ad000003-0000-0000-0000-000000000013', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'chief_complaint', 'Chief Complaint', 'text', true, true, false, 4, 'Clinical'),
('ad000003-0000-0000-0000-000000000014', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'diagnosis', 'Diagnosis', 'textarea', false, false, false, 5, 'Clinical'),
('ad000003-0000-0000-0000-000000000015', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041', 'treatment_plan', 'Treatment Plan', 'longtext', false, false, false, 6, 'Clinical')
ON CONFLICT (entity_type_id, name) DO NOTHING;

-- ============================================================================
-- DEFAULT VIEWS FOR FUTURE MODULES
-- ============================================================================

INSERT INTO view_defs (id, tenant_id, entity_type_id, name, label, view_type, is_default, is_system, columns, filters, sort, settings)
VALUES
-- Appointment views
('ad000004-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030',
 'all_appointments', 'All Appointments', 'table', true, true, '["contact_id", "service_id", "scheduled_at", "status"]'::jsonb, '[]'::jsonb, '[]'::jsonb, '{}'::jsonb),
('ad000004-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000030',
 'appointments_calendar', 'Calendar', 'calendar', false, true, '[]'::jsonb, '[]'::jsonb, '[]'::jsonb, '{"date_field": "scheduled_at", "title_field": "service_id"}'::jsonb),

-- Patient views
('ad000005-0000-0000-0000-000000000001', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000040',
 'all_patients', 'All Patients', 'table', true, true, '["contact_id", "date_of_birth", "blood_type"]'::jsonb, '[]'::jsonb, '[]'::jsonb, '{}'::jsonb),

-- Consultation views
('ad000005-0000-0000-0000-000000000002', 'b128c8da-6e56-485d-b2fe-e45fb7492b2e', 'e0000000-0000-0000-0000-000000000041',
 'all_consultations', 'All Consultations', 'table', true, true, '["patient_id", "doctor_id", "date", "chief_complaint"]'::jsonb, '[]'::jsonb, '[]'::jsonb, '{}'::jsonb)
ON CONFLICT (id) DO NOTHING;
