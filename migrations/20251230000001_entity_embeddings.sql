-- Enable pgvector extension for vector similarity search
CREATE EXTENSION IF NOT EXISTS vector;

-- ============================================================================
-- Entity Embeddings Table
-- Stores vector embeddings for semantic search and RAG (Retrieval-Augmented Generation)
-- ============================================================================

CREATE TABLE IF NOT EXISTS entity_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    
    -- Vector embedding (1536 dimensions for OpenAI ada-002, 384 for all-MiniLM)
    embedding vector(1536),
    
    -- Content that was embedded (for debugging and reprocessing)
    content_hash VARCHAR(64) NOT NULL,  -- SHA256 of the embedded content
    content_preview TEXT,                -- First 500 chars of content
    
    -- Embedding metadata
    model_name VARCHAR(100) NOT NULL DEFAULT 'text-embedding-ada-002',
    model_version VARCHAR(50),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure one embedding per entity (can be updated)
    UNIQUE(tenant_id, entity_id)
);

-- Create indexes for efficient vector search
CREATE INDEX IF NOT EXISTS idx_entity_embeddings_tenant 
    ON entity_embeddings(tenant_id);

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_entity_type 
    ON entity_embeddings(tenant_id, entity_type);

-- HNSW index for fast approximate nearest neighbor search
-- This is the key optimization for vector similarity queries
CREATE INDEX IF NOT EXISTS idx_entity_embeddings_vector 
    ON entity_embeddings 
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- ============================================================================
-- RAG Context Cache
-- Caches frequently accessed context chunks for faster retrieval
-- ============================================================================

CREATE TABLE IF NOT EXISTS rag_context_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    cache_key VARCHAR(255) NOT NULL,
    
    -- Cached context
    context_json JSONB NOT NULL,
    
    -- Cache metadata
    hit_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, cache_key)
);

CREATE INDEX IF NOT EXISTS idx_rag_cache_lookup 
    ON rag_context_cache(tenant_id, cache_key);

CREATE INDEX IF NOT EXISTS idx_rag_cache_expiry 
    ON rag_context_cache(expires_at) 
    WHERE expires_at IS NOT NULL;

-- ============================================================================
-- AI Conversations (for chat history)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Conversation metadata
    title VARCHAR(255),
    context_type VARCHAR(50),  -- 'entity', 'global', 'workflow'
    context_entity_id UUID,    -- Optional: entity this conversation is about
    
    -- Status
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_conversations_user 
    ON ai_conversations(tenant_id, user_id, created_at DESC);

-- ============================================================================
-- AI Messages (conversation messages)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    
    -- Message content
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    
    -- RAG context used for this message
    rag_context JSONB,  -- Stores retrieved entities/documents used
    
    -- Token usage (for billing/monitoring)
    tokens_input INTEGER,
    tokens_output INTEGER,
    model_used VARCHAR(100),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_messages_conversation 
    ON ai_messages(conversation_id, created_at);

-- ============================================================================
-- Helper function: Find similar entities using vector search
-- ============================================================================

CREATE OR REPLACE FUNCTION find_similar_entities(
    p_tenant_id UUID,
    p_query_embedding vector(1536),
    p_entity_type VARCHAR DEFAULT NULL,
    p_limit INTEGER DEFAULT 10,
    p_threshold FLOAT DEFAULT 0.7
)
RETURNS TABLE (
    entity_id UUID,
    entity_type VARCHAR(100),
    similarity FLOAT,
    content_preview TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ee.entity_id,
        ee.entity_type,
        1 - (ee.embedding <=> p_query_embedding) as similarity,
        ee.content_preview
    FROM entity_embeddings ee
    WHERE ee.tenant_id = p_tenant_id
      AND (p_entity_type IS NULL OR ee.entity_type = p_entity_type)
      AND 1 - (ee.embedding <=> p_query_embedding) >= p_threshold
    ORDER BY ee.embedding <=> p_query_embedding
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Trigger to update timestamps
-- ============================================================================

CREATE OR REPLACE FUNCTION update_embedding_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_entity_embeddings_updated
    BEFORE UPDATE ON entity_embeddings
    FOR EACH ROW
    EXECUTE FUNCTION update_embedding_timestamp();

CREATE TRIGGER tr_ai_conversations_updated
    BEFORE UPDATE ON ai_conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_embedding_timestamp();
