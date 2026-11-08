-- ============================================================================
-- Migration 001: Create Users Table
-- ============================================================================
-- Represents customers/account owners in the banking system.
--
-- In Spring Boot / JPA:
--   @Entity
--   @Table(name = "users")
--   public class User {
--       @Id @GeneratedValue private UUID id;
--       @Column(unique = true, nullable = false) private String email;
--       ...
--   }
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_users_email UNIQUE (email)
);

-- Index for lookup by email address (e.g. login, verification)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
