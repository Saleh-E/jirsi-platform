-- Initial schema migration
-- Creates all core tables for the SaaS platform

-- Tenants
CREATE TABLE tenants (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    subdomain VARCHAR(63) NOT NULL UNIQUE,
    custom_domain VARCHAR(255) UNIQUE,
    plan VARCHAR(50) NOT NULL DEFAULT 'free',
    status VARCHAR(50) NOT NULL DEFAULT 'trial',
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenants_subdomain ON tenants(subdomain);
CREATE INDEX idx_tenants_custom_domain ON tenants(custom_domain) WHERE custom_domain IS NOT NULL;

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    avatar_url VARCHAR(500),
    preferences JSONB NOT NULL DEFAULT '{}',
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant_email ON users(tenant_id, email);

-- Teams
CREATE TABLE teams (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE user_teams (
    user_id UUID NOT NULL REFERENCES users(id),
    team_id UUID NOT NULL REFERENCES teams(id),
    is_leader BOOLEAN NOT NULL DEFAULT FALSE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, team_id)
);

-- Sessions
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    token_hash VARCHAR(255) NOT NULL,
    user_agent TEXT,
    ip_address VARCHAR(45),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

-- App definitions
CREATE TABLE app_defs (
    id VARCHAR(50) PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    icon VARCHAR(50),
    description TEXT,
    sort_order INT NOT NULL DEFAULT 0,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Entity types (DocTypes)
CREATE TABLE entity_types (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    app_id VARCHAR(50) NOT NULL,
    module_id VARCHAR(50),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    label_plural VARCHAR(255) NOT NULL,
    icon VARCHAR(50),
    description TEXT,
    flags JSONB NOT NULL DEFAULT '{}',
    default_sort_field VARCHAR(100),
    default_sort_desc BOOLEAN NOT NULL DEFAULT FALSE,
    soft_delete BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_entity_types_tenant_app ON entity_types(tenant_id, app_id);

-- Field definitions
CREATE TABLE field_defs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type_id UUID NOT NULL REFERENCES entity_types(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT FALSE,
    is_unique BOOLEAN NOT NULL DEFAULT FALSE,
    show_in_list BOOLEAN NOT NULL DEFAULT FALSE,
    show_in_card BOOLEAN NOT NULL DEFAULT FALSE,
    is_searchable BOOLEAN NOT NULL DEFAULT FALSE,
    is_filterable BOOLEAN NOT NULL DEFAULT FALSE,
    is_sortable BOOLEAN NOT NULL DEFAULT FALSE,
    is_readonly BOOLEAN NOT NULL DEFAULT FALSE,
    default_value JSONB,
    placeholder VARCHAR(255),
    help_text TEXT,
    validation JSONB,
    options JSONB,
    ui_hints JSONB,
    sort_order INT NOT NULL DEFAULT 0,
    "group" VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(entity_type_id, name)
);

CREATE INDEX idx_field_defs_entity ON field_defs(entity_type_id);

-- View definitions
CREATE TABLE view_defs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type_id UUID NOT NULL REFERENCES entity_types(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    view_type VARCHAR(50) NOT NULL DEFAULT 'table',
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID REFERENCES users(id),
    columns JSONB NOT NULL DEFAULT '[]',
    filters JSONB NOT NULL DEFAULT '[]',
    sort JSONB NOT NULL DEFAULT '[]',
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_view_defs_entity ON view_defs(entity_type_id);

-- Association definitions
CREATE TABLE association_defs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    source_entity VARCHAR(100) NOT NULL,
    target_entity VARCHAR(100) NOT NULL,
    name VARCHAR(100) NOT NULL,
    label_source VARCHAR(255) NOT NULL,
    label_target VARCHAR(255) NOT NULL,
    cardinality VARCHAR(50) NOT NULL,
    source_role VARCHAR(100),
    target_role VARCHAR(100),
    allow_primary BOOLEAN NOT NULL DEFAULT FALSE,
    cascade_delete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Associations (links between records)
CREATE TABLE associations (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    association_def_id UUID NOT NULL REFERENCES association_defs(id),
    source_id UUID NOT NULL,
    target_id UUID NOT NULL,
    role VARCHAR(100),
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_associations_source ON associations(source_id);
CREATE INDEX idx_associations_target ON associations(target_id);

-- Event definitions
CREATE TABLE event_defs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type_id UUID NOT NULL REFERENCES entity_types(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    field_name VARCHAR(100),
    stage_value VARCHAR(100),
    schedule VARCHAR(100),
    description TEXT,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Events (instances)
CREATE TABLE events (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    event_def_id UUID NOT NULL REFERENCES event_defs(id),
    entity_type_id UUID NOT NULL,
    record_id UUID NOT NULL,
    triggered_by UUID REFERENCES users(id),
    payload JSONB NOT NULL DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processing_result JSONB
);

CREATE INDEX idx_events_unprocessed ON events(tenant_id, processed) WHERE processed = FALSE;

-- Node graph definitions
CREATE TABLE node_graph_defs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(255) NOT NULL,
    description TEXT,
    scope VARCHAR(50) NOT NULL,
    graph_type VARCHAR(50) NOT NULL,
    entity_type_id UUID REFERENCES entity_types(id),
    app_id VARCHAR(50),
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Node definitions
CREATE TABLE node_defs (
    id UUID PRIMARY KEY,
    graph_id UUID NOT NULL REFERENCES node_graph_defs(id),
    node_type VARCHAR(50) NOT NULL,
    label VARCHAR(255) NOT NULL,
    x REAL NOT NULL DEFAULT 0,
    y REAL NOT NULL DEFAULT 0,
    config JSONB NOT NULL DEFAULT '{}',
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_node_defs_graph ON node_defs(graph_id);

-- Edge definitions
CREATE TABLE edge_defs (
    id UUID PRIMARY KEY,
    graph_id UUID NOT NULL REFERENCES node_graph_defs(id),
    source_node_id UUID NOT NULL REFERENCES node_defs(id),
    source_port VARCHAR(100) NOT NULL,
    target_node_id UUID NOT NULL REFERENCES node_defs(id),
    target_port VARCHAR(100) NOT NULL,
    label VARCHAR(255)
);

CREATE INDEX idx_edge_defs_graph ON edge_defs(graph_id);

-- Graph executions
CREATE TABLE graph_executions (
    id UUID PRIMARY KEY,
    graph_id UUID NOT NULL REFERENCES node_graph_defs(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    trigger_event_id UUID REFERENCES events(id),
    trigger_record_id UUID,
    status VARCHAR(50) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    error TEXT,
    log JSONB NOT NULL DEFAULT '{}'
);

-- Job queue
CREATE TABLE job_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    job_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority INT NOT NULL DEFAULT 0,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_job_queue_pending ON job_queue(status, priority, created_at) WHERE status = 'pending';

-- CRM: Contacts
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    company_id UUID,
    job_title VARCHAR(255),
    lifecycle_stage VARCHAR(50) NOT NULL DEFAULT 'lead',
    lead_source VARCHAR(100),
    owner_id UUID REFERENCES users(id),
    tags TEXT[] NOT NULL DEFAULT '{}',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_contacts_tenant ON contacts(tenant_id);
CREATE INDEX idx_contacts_email ON contacts(tenant_id, email) WHERE email IS NOT NULL;
CREATE INDEX idx_contacts_name ON contacts(tenant_id, first_name, last_name);

-- CRM: Companies
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255),
    industry VARCHAR(100),
    size VARCHAR(50),
    phone VARCHAR(50),
    website VARCHAR(500),
    address TEXT,
    city VARCHAR(100),
    country VARCHAR(100),
    owner_id UUID REFERENCES users(id),
    tags TEXT[] NOT NULL DEFAULT '{}',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_companies_tenant ON companies(tenant_id);
CREATE INDEX idx_companies_name ON companies(tenant_id, name);

-- Add foreign key for contacts.company_id
ALTER TABLE contacts ADD CONSTRAINT fk_contacts_company
    FOREIGN KEY (company_id) REFERENCES companies(id);

-- CRM: Pipelines
CREATE TABLE pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    stages JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- CRM: Deals
CREATE TABLE deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    amount BIGINT,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    stage VARCHAR(100) NOT NULL,
    pipeline_id UUID NOT NULL REFERENCES pipelines(id),
    probability INT,
    expected_close_date DATE,
    actual_close_date DATE,
    contact_id UUID REFERENCES contacts(id),
    company_id UUID REFERENCES companies(id),
    owner_id UUID REFERENCES users(id),
    lost_reason TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    custom_fields JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_deals_tenant ON deals(tenant_id);
CREATE INDEX idx_deals_stage ON deals(tenant_id, stage);
CREATE INDEX idx_deals_pipeline ON deals(pipeline_id);

-- CRM: Tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    due_date TIMESTAMPTZ,
    priority VARCHAR(50) NOT NULL DEFAULT 'normal',
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    task_type VARCHAR(50) NOT NULL DEFAULT 'todo',
    linked_entity_type VARCHAR(100),
    linked_entity_id UUID,
    assignee_id UUID REFERENCES users(id),
    created_by UUID NOT NULL REFERENCES users(id),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_tenant ON tasks(tenant_id);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_linked ON tasks(linked_entity_type, linked_entity_id);

-- Interactions (activity log)
CREATE TABLE interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_type VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    interaction_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_minutes INT,
    outcome VARCHAR(100),
    attachments UUID[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_interactions_record ON interactions(entity_type, record_id);
CREATE INDEX idx_interactions_occurred ON interactions(tenant_id, occurred_at);

-- Interaction participants
CREATE TABLE interaction_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    interaction_id UUID NOT NULL REFERENCES interactions(id),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    role VARCHAR(100)
);

CREATE INDEX idx_interaction_participants ON interaction_participants(interaction_id);
