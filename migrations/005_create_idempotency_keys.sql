-- ============================================================================
-- Migration 005: Create Idempotency Keys Table
-- ============================================================================
-- Protects write endpoints (deposit, withdraw, transfer) from accidental
-- duplicate executions caused by client retries or network drops.
--
-- Mechanics:
-- 1. Client supplies an `Idempotency-Key` header with each mutating request.
-- 2. `request_hash` stores the SHA-256 digest of the normalized request payload.
-- 3. If an identical request arrives with the same key, the previously recorded
--    `response_status` and `response_body` are returned without re-executing.
-- 4. If the key is reused with a DIFFERENT payload, the system rejects it (409 Conflict).
-- ============================================================================

CREATE TABLE IF NOT EXISTS idempotency_keys (
    id UUID PRIMARY KEY,
    key VARCHAR(255) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    transaction_id UUID,
    response_status SMALLINT NOT NULL,
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_idempotency_keys_key UNIQUE (key),
    CONSTRAINT fk_idempotency_keys_transaction FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE SET NULL
);

-- Fast lookup for checking idempotency key existence
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_key ON idempotency_keys(key);
-- Index for lifecycle management (e.g., purging stale keys after TTL retention period)
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_created_at ON idempotency_keys(created_at);
