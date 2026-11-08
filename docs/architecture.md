# Ledger Service — Architecture

This document explains the architecture and key design decisions of the Ledger service.

## Design Philosophy

Ledger follows **Clean Architecture** (also known as Hexagonal Architecture or Ports and Adapters), adapted for idiomatic Rust. The goal is to keep business logic independent of frameworks, databases, and HTTP concerns.

### For Java/Spring Boot Developers

If you're coming from Spring Boot, here's how the concepts map:

| Spring Boot Concept | Rust/Ledger Equivalent |
|---|---|
| `@SpringBootApplication` | `main.rs` + `bootstrap.rs` |
| `@RestController` | Axum handler functions |
| `@Service` | Application service structs |
| `@Repository` (interface) | Rust traits in `domain/*/repository.rs` |
| `@Repository` (impl) | Structs in `infrastructure/repositories/` |
| `@Autowired` / IoC Container | Constructor injection via `AppState` |
| `application.yml` | Environment variables + `config/settings.rs` |
| `@ControllerAdvice` | `impl IntoResponse for AppError` |
| Bean Validation (`@Valid`) | `validator` crate with `#[derive(Validate)]` |
| `ResponseEntity<T>` | `ApiResponse<T>` wrapper |
| Spring Actuator | Custom `/health` and `/health/ready` endpoints |
| Logback / SLF4J | `tracing` + `tracing-subscriber` |
| Jackson | Serde |
| `BigDecimal` | `rust_decimal::Decimal` |
| JPA/Hibernate | SQLx (no ORM — direct SQL) |

## Layer Architecture

### 1. Domain Layer (`src/domain/`)

The innermost layer. Contains:

- **Entities**: Core business objects (`Account`, `Transaction`, `LedgerEntry`)
- **Value Objects**: Immutable types with validation (`Money`, `AccountNumber`, `Currency`)
- **Repository Traits**: Interfaces (Rust traits) that define data access contracts

This layer has **zero dependencies** on external crates (except standard types like `uuid`, `chrono`, `rust_decimal`). It never imports Axum, SQLx, or any infrastructure code.

**Rust concept — Traits as interfaces**: In Java, you define `interface AccountRepository`. In Rust, you define `trait AccountRepository`. The key difference is that Rust traits can be used both as static dispatch (generics, monomorphization — zero-cost) and dynamic dispatch (`dyn Trait` — like Java interfaces with vtable lookup).

### 2. Application Layer (`src/application/`)

Contains use cases (services) that orchestrate domain objects:

- **Commands**: Input types for write operations (create account, transfer money)
- **Queries**: Input types for read operations (get account, list transactions)
- **Services**: Orchestrate domain logic, enforce business rules, coordinate repositories

Services accept repository traits (not concrete implementations), enabling testability.

### 3. Infrastructure Layer (`src/infrastructure/`)

Implements external concerns:

- **Database**: Connection pool management (SQLx + PostgreSQL)
- **Repositories**: Concrete implementations of domain repository traits
- **External**: Outbound HTTP client (Reqwest) for external service integration

**Rust concept — Trait objects for dependency injection**: Instead of Spring's runtime DI container, we use `Arc<dyn AccountRepository>` to inject repository implementations. The compiler ensures type safety at compile time.

### 4. Interfaces Layer (`src/interfaces/`)

Handles external input/output:

- **HTTP Handlers**: Axum request handlers (extract request, call service, return response)
- **DTOs**: Request/response data transfer objects (separate from domain entities)
- **Middleware**: Cross-cutting concerns (request ID, logging, CORS)
- **Routes**: URL routing configuration

**Rust concept — Axum extractors**: Instead of Spring's `@RequestBody`, `@PathVariable`, `@RequestParam`, Axum uses the type system. If a handler accepts `Json<CreateAccountRequest>`, Axum automatically deserializes the request body. If deserialization fails, it returns a 400 error — no manual parsing needed.

## Dependency Rule

```
Interfaces → Application → Domain ← Infrastructure
```

- Inner layers never depend on outer layers
- Domain defines traits; Infrastructure implements them
- Application orchestrates; it doesn't know about HTTP or SQL
- Interfaces translate between HTTP and application commands/queries

## Error Handling Strategy

Three levels of errors:

1. **Domain errors** (`thiserror`): Typed, specific errors like `InsufficientBalance`
2. **Application errors**: Wrapped domain errors with additional context
3. **HTTP errors**: Converted to appropriate status codes and JSON via `IntoResponse`

This ensures:
- Domain logic doesn't know about HTTP status codes
- Infrastructure errors (SQL, network) are never leaked to API clients
- Every error is logged with appropriate severity and context

## Configuration

Environment-variable driven configuration using `dotenvy`:

- **Development**: `.env` file loaded automatically
- **Production**: Environment variables set by container orchestrator
- **Defaults**: Sensible defaults for all non-secret settings
- `DATABASE_URL` is the only required variable (no default with real credentials)

This follows the [12-Factor App](https://12factor.net/config) methodology.

## Observability

Structured JSON logging via `tracing`:

- Every HTTP request gets a unique `x-request-id` (UUIDv7, time-ordered)
- Request/response logging with method, URI, status code, and latency
- Transaction processing events with correlation IDs
- Error logging with full context (but never sensitive data)
- `RUST_LOG` environment variable controls log levels

The logging infrastructure is designed for future OpenTelemetry integration — `tracing` is the standard Rust telemetry facade that OpenTelemetry exporters plug into.

## Financial Data Integrity

*(Detailed in Phase 4)*

- Double-entry accounting: every transaction produces balanced debit/credit ledger entries
- PostgreSQL transactions for atomicity
- Row-level locking (`SELECT ... FOR UPDATE`) to prevent race conditions
- Deterministic lock ordering to prevent deadlocks
- `rust_decimal::Decimal` for exact monetary arithmetic (never floating point)
- Idempotency keys to prevent duplicate money movement
