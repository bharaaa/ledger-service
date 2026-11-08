# Ledger Service

A modern banking transaction service built in Rust, designed to manage bank accounts, process financial transactions, maintain account balances, and record immutable financial ledger entries.

## Technology Stack

| Category | Technology |
|---|---|
| Language | Rust (edition 2024) |
| Async Runtime | Tokio |
| HTTP Framework | Axum 0.8 |
| Serialization | Serde |
| Database | PostgreSQL 16 |
| Database Access | SQLx (compile-time checked queries) |
| HTTP Client | Reqwest |
| API Documentation | Utoipa / OpenAPI 3.1 |
| Observability | tracing + tracing-subscriber (structured JSON) |
| Configuration | dotenvy (environment variables) |
| Error Handling | thiserror + anyhow |
| Validation | validator |
| Decimal Handling | rust_decimal (never f32/f64 for money) |
| Testing | cargo test + testcontainers |
| Containerization | Docker + Docker Compose |

## Architecture

The project follows Clean Architecture / Hexagonal Architecture principles with clear layer boundaries:

```
┌─────────────────────────────────────────────────────────────┐
│                    Interfaces Layer                         │
│   HTTP handlers, DTOs, middleware, routes                   │
├─────────────────────────────────────────────────────────────┤
│                   Application Layer                         │
│   Services, commands, queries (use cases)                   │
├─────────────────────────────────────────────────────────────┤
│                     Domain Layer                            │
│   Entities, value objects, repository traits                │
├─────────────────────────────────────────────────────────────┤
│                  Infrastructure Layer                       │
│   Database pool, repository impls, external HTTP client     │
└─────────────────────────────────────────────────────────────┘
```

See [docs/architecture.md](docs/architecture.md) for detailed design decisions.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.85+)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- PostgreSQL 16 (via Docker or local installation)

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd ledger-service
   ```

2. **Create environment file**
   ```bash
   cp .env.example .env
   ```

3. **Start PostgreSQL**
   ```bash
   docker compose up -d postgres
   ```

4. **Run database migrations** (Phase 2+)
   ```bash
   sqlx database create
   sqlx migrate run
   ```

5. **Run the application**
   ```bash
   cargo run
   ```

   The server starts at `http://localhost:8080`.

### Health Check

```bash
# Liveness
curl http://localhost:8080/health

# Readiness
curl http://localhost:8080/health/ready
```

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# All checks
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `postgres://postgres:postgres@localhost:5432/ledger` | PostgreSQL connection string |
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `DATABASE_MAX_CONNECTIONS` | `10` | Max database pool connections |
| `DATABASE_MIN_CONNECTIONS` | `2` | Min database pool connections |
| `DATABASE_ACQUIRE_TIMEOUT_SECONDS` | `5` | Connection acquire timeout |
| `RUST_LOG` | `info` | Log level filter |

### API Endpoints

| Method | Path | Description | Phase |
|---|---|---|---|
| GET | `/health` | Liveness check | 1 |
| GET | `/health/ready` | Readiness check | 1 |
| POST | `/api/v1/accounts` | Create account | 3 |
| GET | `/api/v1/accounts` | List accounts | 3 |
| GET | `/api/v1/accounts/{id}` | Get account | 3 |
| GET | `/api/v1/accounts/{id}/balance` | Get balance | 3 |
| GET | `/api/v1/accounts/{id}/transactions` | Account history | 3 |
| POST | `/api/v1/transactions/deposit` | Deposit | 4 |
| POST | `/api/v1/transactions/withdraw` | Withdraw | 4 |
| POST | `/api/v1/transactions/transfer` | Transfer | 4 |
| GET | `/api/v1/transactions/{id}` | Get transaction | 4 |
| GET | `/api/v1/transactions` | List transactions | 4 |

## License

MIT
