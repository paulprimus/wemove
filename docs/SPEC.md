# WeMove Web Application

## Overview

A web application built with Tokio, Axum, and Clap providing a HelloWorld endpoint with health check.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Tokio |
| Web Framework | Axum |
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
│   ├── common/             # Shared types, errors, tracing setup
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs    # Application error types
│   │       └── tracing.rs  # Tracing setup
│   ├── config/             # Configuration loading
│   │   ├── Cargo.toml
│   │   └── src/
│   │   └── lib.rs      # CLI + env config
│   └── server/             # Axum HTTP server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── routes.rs    # Route definitions
│           ├── handlers.rs # Endpoint handlers
│           └── middleware.rs # Request logging
├── web/                    # Angular frontend (CSR, standalone components)
│   ├── angular.json
│   ├── package.json
│   ├── proxy.conf.json     # Dev-server proxy to the Rust backend (/api -> :8080)
│   └── src/
│       ├── main.ts
│       └── app/
└── tests/
    └── integration_tests.rs
```

## Crate Responsibilities

### common
- Tracing subscriber initialization
- Application error types (`thiserror`)
- Shared DTOs (HelloWorldRequest, HelloWorldResponse)

### config
- Clap CLI argument parsing
- Environment variable loading via dotenvy
- Config precedence: CLI > ENV > defaults

### server
- Axum router setup
- Middleware (request logging, metrics)
- Endpoint handlers
- Health check endpoint

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

### GET /
HelloWorld endpoint. Returns a greeting.

**Response (200):**
```json
{
  "message": "Hello, World!"
}
```

### POST /
HelloWorld endpoint with JSON body support.

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

### config
- `clap` (derive, env)
- `dotenvy`

### server
- `tokio` (full)
- `axum`
- `axum-prometheus`
- `anyhow`
- `serde_json`

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

## Frontend (web/)

Angular application (client-side rendering, standalone components, routing enabled).

### Run dev server
```bash
cd web
npm start
```
Runs `ng serve` with `proxy.conf.json`, forwarding requests under `/api/*` to the
Rust backend at `http://127.0.0.1:8080` (prefix stripped). Start the backend
separately with `cargo run --package server`.

### Build for production
```bash
cd web
npm run build
```
Output is written to `web/dist/web`.

### Test
```bash
cd web
npm test
```