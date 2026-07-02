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
| Configuration | Clap + Env + TOML |

## Project Structure

```
wemove/
├── Cargo.toml              # Workspace root
├── .env                    # Environment defaults
├── config.toml             # Config file template
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
│   │       └── lib.rs      # CLI + env + file config
│   └── server/             # Axum HTTP server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── routes.rs    # Route definitions
│           ├── handlers.rs # Endpoint handlers
│           └── middleware.rs # Request logging
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
- Environment variable loading
- TOML config file loading
- Config precedence: CLI > ENV > config.toml

### server
- Axum router setup
- Middleware (request logging, metrics)
- Endpoint handlers
- Health check endpoint

## Configuration

### Precedence
CLI arguments > Environment variables > config.toml > defaults

### Options

| Option | CLI | Env | Config | Default |
|--------|-----|-----|--------|---------|
| Host | `--host` | `HOST` | `server.host` | `127.0.0.1` |
| Port | `--port` | `PORT` | `server.port` | `8080` |
| Log Level | `--log-level` | `RUST_LOG` | `logging.level` | `info` |

### Example config.toml

```toml
[server]
host = "0.0.0.0"
port = 8080

[logging]
level = "debug"
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
- `config`
- `serde`

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

### Config file
```bash
cp config.toml.example config.toml
# Edit config.toml
cargo run --package server
```

### Test
```bash
cargo test --workspace
```