-- ============================================================================
-- Migration 002: Create Accounts Table
-- ============================================================================
-- Represents customer bank accounts.
--
-- Financial Data Integrity Notes:
-- 1. `balance` is stored as NUMERIC(20, 4) — never floating point.
-- 2. CHECK (balance >= 0.0000) guarantees at the database engine level
--    that account balances can NEVER go negative (no unauthorized overdraft).
-- 3. CHECK (status IN ('ACTIVE', 'BLOCKED', 'CLOSED')) restricts valid states.
-- 4. Foreign key to `users(id)` with ON DELETE RESTRICT prevents accidental
--    deletion of customers who own accounts.
-- ============================================================================

CREATE TABLE IF NOT EXISTS accounts (
    id UUID PRIMARY KEY,
    account_number VARCHAR(32) NOT NULL,
    customer_id UUID NOT NULL,
    account_name VARCHAR(255) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    balance NUMERIC(20, 4) NOT NULL DEFAULT 0.0000,
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_accounts_account_number UNIQUE (account_number),
    CONSTRAINT fk_accounts_customer FOREIGN KEY (customer_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT chk_accounts_status CHECK (status IN ('ACTIVE', 'BLOCKED', 'CLOSED')),
    CONSTRAINT chk_accounts_balance_non_negative CHECK (balance >= 0.0000),
    CONSTRAINT chk_accounts_currency_length CHECK (length(currency) = 3)
);

-- Indexes for frequent queries:
-- Fast lookup by account number for transfers and balance inquiries
CREATE INDEX IF NOT EXISTS idx_accounts_account_number ON accounts(account_number);
-- Fast listing of accounts owned by a specific customer
CREATE INDEX IF NOT EXISTS idx_accounts_customer_id ON accounts(customer_id);
-- Fast filtering by account operational status
CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts(status);
