-- ============================================================================
-- Migration 004: Create Ledger Entries Table
-- ============================================================================
-- The immutable financial ledger — the core source of truth for the entire service.
--
-- Accounting Rules:
-- 1. Double-Entry Invariant:
--    For every financial transaction, the sum of DEBITs must equal the sum of CREDITs.
-- 2. Append-Only Immutability:
--    Financial records are never modified (UPDATE) or deleted (DELETE).
--    Mistakes or refunds are corrected through new compensating reversal transactions.
-- 3. Strict Check Constraints:
--    `entry_type` is restricted to 'DEBIT' or 'CREDIT'.
--    `amount` must be strictly positive.
-- ============================================================================

CREATE TABLE IF NOT EXISTS ledger_entries (
    id UUID PRIMARY KEY,
    transaction_id UUID NOT NULL,
    account_id UUID NOT NULL,
    entry_type VARCHAR(10) NOT NULL,
    amount NUMERIC(20, 4) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT fk_ledger_entries_transaction FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE RESTRICT,
    CONSTRAINT fk_ledger_entries_account FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    CONSTRAINT chk_ledger_entries_type CHECK (entry_type IN ('DEBIT', 'CREDIT')),
    CONSTRAINT chk_ledger_entries_amount_positive CHECK (amount > 0.0000),
    CONSTRAINT chk_ledger_entries_currency_length CHECK (length(currency) = 3)
);

-- Fast lookup of all entries for a specific transaction (audit balance verification)
CREATE INDEX IF NOT EXISTS idx_ledger_entries_transaction_id ON ledger_entries(transaction_id);
-- Fast lookup of all entries impacting a specific account (reconciliation)
CREATE INDEX IF NOT EXISTS idx_ledger_entries_account_id ON ledger_entries(account_id);
-- Chronological index for audit logs and statement generation
CREATE INDEX IF NOT EXISTS idx_ledger_entries_created_at ON ledger_entries(created_at DESC);
