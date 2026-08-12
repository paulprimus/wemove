# WeMove Web Application

## Overview

A full-stack web application built with Tokio, Axum, Topcoat, and Clap providing a Main endpoint with health check.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Tokio |
| Web Framework | Axum (server) |
| Frontend Framework | Topcoat (SSR, full-stack Rust) |
| CLI | Clap |
| Logging | Tracing |
| Configuration | Clap + Env (.env) |

## Project Structure

```
wemove/
├── Cargo.toml              # Workspace root
├── .env                    # Environment variables
├── docs/
│   └── SPEC.md             # This specification
├── crates/
│   ├── common/             # Shared types, errors, tracing setup (framework-agnostic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs    # AppError enum (thiserror only, no HTTP mapping)
│   │       └── tracing.rs  # Tracing setup
│   ├── config/             # Configuration loading
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs      # CLI + env config
│   ├── server/             # Axum HTTP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes.rs    # Route definitions
│   │       ├── handlers.rs # Endpoint handlers (utoipa::path annotations)
│   │       ├── error.rs    # ApiError newtype wrapping AppError, impl IntoResponse
│   │       ├── openapi.rs  # OpenAPI spec (utoipa::OpenApi)
│   │       └── auth_rest.rs
│   ├── web/                # Topcoat full-stack frontend (SSR)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs      # Pages and components (registered via topcoat)
│   └── web-server/         # Optional: dedicated Topcoat web server (standalone dev)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── tests/
    └── integration_tests.rs
```

## Crate Responsibilities

### common
- Tracing subscriber initialization
- Application error types (`thiserror`), framework-agnostic (kein Axum, kein `anyhow` als
  Pflicht-Dependency — `AppError::Internal` trägt eine `String`-Message; die Konvertierung
  aus `anyhow::Error` erfolgt im aufrufenden Code)
- Shared DTOs (MainRequest, MainResponse, HealthResponse), annotiert mit
  `utoipa::ToSchema` für die OpenAPI-Generierung

### config
- Clap CLI argument parsing
- Environment variable loading via dotenvy
- Config precedence: CLI > ENV > defaults

### server
- Axum router setup
- Middleware (request logging, metrics)
- Endpoint handlers
- Health check endpoint
- `ApiError`-Newtype (`error.rs`), das `common::AppError` in eine HTTP-`Response`
  übersetzt (`IntoResponse`). Die HTTP-Mapping-Logik lebt bewusst hier und nicht in
  `common`, um `common` framework-agnostisch zu halten (siehe Orphan-Rule-Hinweis im Code)

### web
- [Topcoat](https://github.com/tokio-rs/topcoat) full-stack frontend
- Server-Side Rendering (SSR) mit dem `view!`-Makro
- Pages (`#[page]`) und Components (`#[component]`) in `lib.rs`
- Client-Reactivity via `$(...)`-Expressions (Rust + JS)
- Routing via Topcoat Router (`RouterBuilder`), module-based routing möglich
- Pages werden über `register()` im `RouterBuilder` registriert
- Optional integrierbar in den Axum-Server oder als eigenständiger Server

## Configuration

### Precedence
CLI arguments > Environment variables > .env file > defaults

### Options

| Option | CLI | ENV | Default |
|--------|-----|-----|---------|
| Host | `--host` | `HOST` | `127.0.0.1` |
| Port | `--port` | `PORT` | `8080` |
| Log Level | `--log-level` | `RUST_LOG` | `info` |

### Example .env

```env
HOST=0.0.0.0
PORT=8080
RUST_LOG=debug
```

## Endpoints

Interaktive API-Dokumentation: Swagger-UI unter `/swagger-ui/`, OpenAPI-Spec als JSON unter `/api-docs/openapi.json`.

### GET /api/main
Main endpoint. Returns a greeting.

**Response (200):**
```json
{
  "message": "Hello, WeMove!"
}
```

### POST /api/main
Main endpoint with JSON body support.

**Request:**
```json
{
  "name": "Alice"
}
```

**Response (200):**
```json
{
  "message": "Hello, Alice!"
}
```

### GET /health
Health check endpoint for liveness probes.

**Response (200):**
```json
{
  "status": "healthy"
}
```

## Middleware

- **Request Logging**: All incoming requests logged with method, path, status, and duration
- **Prometheus Metrics**: HTTP request metrics exported at `/metrics`
- **Swagger-UI**: OpenAPI documentation at `/swagger-ui/`, spec at `/api-docs/openapi.json`

## OpenAPI / Swagger

Die API ist mit [utoipa](https://github.com/juhaku/utoipa) als OpenAPI-Spezifikation dokumentiert.
Single Source of Truth ist der Rust-Code – die Spec wird zur Compilezeit generiert.

### Endpoints
| URL | Beschreibung |
|-----|-------------|
| `/api-docs/openapi.json` | OpenAPI 3.0 Spec als JSON |
| `/swagger-ui/` | Interaktive Swagger-UI |

### Generierung
DTOs in `crates/common/src/lib.rs` sind mit `#[derive(ToSchema)]` annotiert.
Handler in `crates/server/src/handlers.rs` sind mit `#[utoipa::path(...)]` versehen.
Das `ApiDoc`-Struct in `crates/server/src/openapi.rs` sammelt alle Pfade und Schemas.

### Error-Handling-Pattern
`common::AppError` ist ein reines `thiserror`-Enum ohne Web-Framework-Bezug (auch mit
`ToSchema` annotiert, damit Fehlerfälle in der OpenAPI-Spec dokumentiert werden können).
Die Übersetzung in eine Axum-`Response` übernimmt `server::error::ApiError`, ein
Newtype-Wrapper mit `impl IntoResponse`. Dadurch bleibt `common` unabhängig von Axum und
wiederverwendbar für andere Consumer (CLI, andere Web-Layer, Tests).

### Topcoat Frontend
Das Frontend ist ein Rust-Crate (`crates/web`) – kein separater TypeScript-Client nötig.
Pages und Components sind typsicherer Rust-Code, der direkt auf dem Server rendert.

## Testing

### Unit Tests
Each crate contains unit tests for its components.

### Integration Tests
`tests/integration_tests.rs` contains integration tests covering:
- All endpoints
- Configuration loading
- Error handling

## Dependencies

### common
- `tracing`
- `tracing-subscriber`
- `thiserror`
- `serde`
- `utoipa`

### config
- `clap` (derive, env)
- `dotenvy`

### server
- `tokio` (full)
- `axum`
- `metrics` + `metrics-exporter-prometheus`
- `anyhow`
- `serde_json`
- `utoipa`
- `utoipa-swagger-ui` (axum feature)

### web
- `topcoat`
- `tokio`

## Usage

### Run
```bash
cargo run --package server
```

### Run with CLI args
```bash
cargo run --package server -- --host 0.0.0.0 --port 3000 --log-level debug
```

### Environment
```bash
export HOST=0.0.0.0
export PORT=3000
export RUST_LOG=debug
cargo run --package server
```

### Test
```bash
cargo test --workspace
```

## Frontend (crates/web)

[Topcoat](https://github.com/tokio-rs/topcoat) full-stack frontend mit Server-Side Rendering.

### Struktur

- `crates/web/src/lib.rs`: Pages und Components, registriert via `register()`
- Pages mit `#[page("/path")]`
- Components mit `#[component]`

### Run

```bash
cargo run --package web-server
```
oder via Topcoat CLI (`topcoat run`).

### Topcoat CLI
```bash
cargo install topcoat-cli
topcoat fmt      # Formatiert view!-Makros
topcoat ui       # Kopiert UI-Komponenten ins Projekt
```