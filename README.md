# WeMove

Web application with Rust/Axum backend and Angular frontend.

## Quick Start

### Backend

```bash
# Run server
cargo run --package server

# With custom config
cargo run --package server -- --host 0.0.0.0 --port 3000 --log-level debug

# Run tests
cargo test --workspace
```

### Frontend

```bash
cd web
npm install
npm start
```

Frontend wird unter `http://localhost:4200` bereitgestellt und leitet `/api/*` Requests an den Backend-Server auf Port 8080 weiter.

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/main` | Main endpoint (GET) |
| `POST /api/main` | Main endpoint mit JSON body |
| `GET /api/health` | Health check |
| `GET /metrics` | Prometheus metrics |

## Projektstruktur

```
wemove/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── common/         # Shared: tracing, error types
│   ├── config/         # CLI + env config
│   └── server/         # Axum HTTP server
├── web/                # Angular frontend
├── tests/              # Integration tests
└── docs/SPEC.md        # Detailed specification
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Tokio, Axum |
| Config | Clap, dotenvy |
| Monitoring | metrics, prometheus |
| Frontend | Angular 22 |
| Logging | tracing, tracing-subscriber |

## Konfiguration

Reihenfolge: CLI args > ENV > .env > defaults

| Option | CLI | ENV | Default |
|--------|-----|-----|---------|
| Host | `--host` | `HOST` | `127.0.0.1` |
| Port | `--port` | `PORT` | `8080` |
| Log Level | `--log-level` | `RUST_LOG` | `info` |

Siehe `docs/SPEC.md` fuer weitere Details.