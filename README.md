# WeMove

Web application with a Topcoat frontend and Axum REST backend.

## Quick Start

### Backend

```bash
# Build the binary and its Topcoat asset bundle
topcoat asset bundle --package server

# Run server
cargo run --package server

# With custom config
cargo run --package server -- --host 0.0.0.0 --port 3000 --log-level debug

# Run tests
cargo test --workspace
```

### Web application

```bash
# Install once (matching the workspace's Topcoat version)
cargo install topcoat-cli@0.7 --locked

# Build Tailwind, bundle assets, and run the server
topcoat dev
```

The Tailwind build uses the locally installed npm CLI from
`crates/web/node_modules/.bin/tailwindcss`. The project-local Cargo configuration
sets `TAILWINDCSS` automatically:

```bash
npm install
cargo build -p web
```

This avoids the automatic GitHub download performed by Topcoat's default
configuration and is useful in environments with TLS-inspecting proxies.

For a manual server start, create the asset bundle after building:

```bash
topcoat asset bundle --package server
cargo run --package server
```

Topcoat rendert die Weboberfläche zusammen mit dem Server unter
`http://localhost:8080`. Die Axum-API bleibt unter `/api/*` verfügbar.

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
│   ├── server/         # Axum API und Topcoat-Server
│   └── web/            # Topcoat-Seiten und Komponenten
├── tests/              # Integration tests
└── docs/SPEC.md        # Detailed specification
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Tokio, Axum |
| Config | Clap, dotenvy |
| Monitoring | metrics, prometheus |
| Frontend | Rust, Topcoat |
| Logging | tracing, tracing-subscriber |

## Konfiguration

Reihenfolge: CLI args > ENV > .env > defaults

| Option | CLI | ENV | Default |
|--------|-----|-----|---------|
| Host | `--host` | `HOST` | `127.0.0.1` |
| Port | `--port` | `PORT` | `8080` |
| Log Level | `--log-level` | `RUST_LOG` | `info` |

Siehe `docs/SPEC.md` fuer weitere Details.
