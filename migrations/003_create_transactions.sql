-- ============================================================================
-- Migration 003: Create Transactions Table
-- ============================================================================
-- Records all financial transaction requests and their high-level lifecycle.
--
-- Financial Data Integrity Notes:
-- 1. `amount > 0.0000`: Zero or negative transaction amounts are rejected.
-- 2. Anti-self-transfer constraint ensures source and destination cannot be
--    the same account when both are populated.
-- 3. At least one of source or destination must be provided.
-- 4. Foreign keys to `accounts(id)` prevent transactions against non-existent accounts.
-- ============================================================================

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY,
    reference_number VARCHAR(64) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL,
    source_account_id UUID,
    destination_account_id UUID,
    amount NUMERIC(20, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMPTZ,

    CONSTRAINT uq_transactions_reference_number UNIQUE (reference_number),
    CONSTRAINT fk_transactions_source_account FOREIGN KEY (source_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT fk_transactions_dest_account FOREIGN KEY (destination_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_transactions_amount_positive CHECK (amount > 0.0000),
    CONSTRAINT chk_transactions_type CHECK (transaction_type IN ('DEPOSIT', 'WITHDRAWAL', 'TRANSFER')),
    CONSTRAINT chk_transactions_status CHECK (status IN ('PENDING', 'COMPLETED', 'FAILED')),
    CONSTRAINT chk_transactions_account_present CHECK (source_account_id IS NOT NULL OR destination_account_id IS NOT NULL),
    CONSTRAINT chk_transactions_distinct_accounts CHECK (
        source_account_id IS NULL OR
        destination_account_id IS NULL OR
        source_account_id != destination_account_id
    )
);

-- Indexes for querying transaction history:
-- Fast lookup by sender account
CREATE INDEX IF NOT EXISTS idx_transactions_source_account_id ON transactions(source_account_id);
-- Fast lookup by receiver account
CREATE INDEX IF NOT EXISTS idx_transactions_dest_account_id ON transactions(destination_account_id);
-- Fast time-based filtering and sorting for account statements
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at DESC);
-- Filter transactions by operational status (e.g. audit pending or failed)
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
