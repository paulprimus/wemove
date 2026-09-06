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
│   ├── SPEC.md             # This specification
│   ├── auth_ablauf.md       # Authentication flow documentation
│   └── openapi_plan.md     # OpenAPI integration plan
├── crates/
│   ├── common/             # Shared types, errors, tracing setup (framework-agnostic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs      # Public exports, DTOs with ToSchema
│   │       └── error.rs    # AppError + DbError enums (thiserror, ToSchema)
│   ├── config/             # Configuration loading
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs      # Args + AuthConfig (clap, dotenvy)
│   ├── server/             # Axum HTTP server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs     # Entry point
│   │       ├── routes.rs    # Router + middleware (TraceLayer, Prometheus, Auth)
│   │       ├── handlers.rs  # GET/POST /api/main, GET /api/health (utoipa)
│   │       ├── state.rs     # AppState { app_name, db } + migrations
│   │       ├── error.rs     # ApiError newtype wrapping AppError, impl IntoResponse
│   │       ├── openapi.rs   # ApiDoc struct (utoipa::OpenApi)
│   │       ├── auth_rest.rs # REST: /api/auth/login, /api/auth/token, /api/auth/register
│   │       ├── user_repo.rs # UserRepository (find_by_email, create, verify_password)
│   └── web/                # Topcoat full-stack frontend (SSR)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs       # register() function, module declarations
│           ├── app.rs      # Root layout (HTML shell, Topcoat-Tailwind)
│           ├── home.rs     # Page "/" — landing
│           └── auth/
│               ├── login.rs    # Page "/login" — Form POST /api/auth/login
│               └── register.rs # Page "/register" — Form POST /auth/register
```

## Crate Responsibilities

### common
- Tracing subscriber initialization
- Application error types (`thiserror`), framework-agnostic (kein Axum, kein `anyhow` als
  Pflicht-Dependency — `AppError::Internal` trägt eine `String`-Message; die Konvertierung
  aus `anyhow::Error` erfolgt im aufrufenden Code)
- Shared DTOs (MainRequest, MainResponse, HealthResponse, LoginRequest/Response,
  RegisterRequest/Response), annotiert mit `utoipa::ToSchema` für die OpenAPI-Generierung
- `DbError`-Enum für Datenbank-spezifische Fehler (Connection, Query, Constraint, PasswordHash)

### config
- Clap CLI argument parsing
- Environment variable loading via dotenvy
- Config precedence: CLI > ENV > defaults
- `AuthConfig` für JWT_SECRET und token_expiry_secs (nur ENV, kein CLI-Flag)

### server
- Axum router setup
- Middleware (TraceLayer, Prometheus metrics, AuthRouter, AppState)
- Endpoint handlers
- Health check endpoint
- `ApiError`-Newtype (`error.rs`), das `common::AppError` in eine HTTP-`Response`
  übersetzt (`IntoResponse`). Die HTTP-Mapping-Logik lebt bewusst hier und nicht in
  `common`, um `common` framework-agnostisch zu halten (siehe Orphan-Rule-Hinweis im Code)
- SQLite-DB über Turso für User-Storage
- `UserRepository` für find_by_email, create, verify_password (bcrypt)
- DB-Migration: `users`-Tabelle wird beim Start erstellt
- JWT-Erzeugung erfolgt direkt in `state.rs` mit `jsonwebtoken`.

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

### GET /api/health
Health check endpoint for liveness probes (nicht `/health`, sondern `/api/health`).

**Response (200):**
```json
{
  "status": "healthy"
}
```

### POST /api/auth/login
User-Login mit Email + Passwort. Fragt die SQLite-DB (Turso) via `UserRepository` ab, verifiziert
das Passwort mit bcrypt und generiert ein JWT via `AppState::create_access_token()`.
Form-Submit (`application/x-www-form-urlencoded`).

**Request (Form):**
```
email=alice@example.com&password=geheim
```

**Response (200):**
```json
{
  "success": true,
  "message": "Login successful",
  "token": "eyJhbGciOiJIUzI1NiJ9..."
}
```

### POST /api/auth/token
OAuth2-ähnlicher Token-Endpunkt. Generiert ein JWT (HS256) ohne User-Authentifizierung.
Nimmt `client_id` und optional `scope` als Formulardaten entgegen.

**Request (Form):**
```
client_id=mein-client&scope=read%20write
```

Inhaltlich entspricht das:
```json
{
  "client_id": "mein-client",
  "scope": "read write"
}
```

**Response (200):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read write"
}
```

### POST /api/auth/register
REST-User-Registrierung. Erstellt einen neuen User in der SQLite-DB mit bcrypt-gehashtem Passwort.
Form-Submit (`application/x-www-form-urlencoded`).

**Request (Form):**
```
name=Alice&email=alice@example.com&password=geheim
```

**Response (201):**
```json
{
  "success": true,
  "message": "Registration successful",
  "user_id": 1
}
```

### GET /metrics
Prometheus-Metriken (Counter, Histogram) für Request-Zählung und Latenz.

### GET /swagger-ui/
Interaktive Swagger-UI. OpenAPI-JSON unter `/api-docs/openapi.json`.

### Web-Authentifizierung
Das Topcoat-Frontend stellt zusätzlich Formularrouten bereit:

| Pfad | Methode | Beschreibung |
|------|---------|-------------|
| `/login` | GET | Login-Seite |
| `/auth/login` | POST | Login-Formular, Weiterleitung zum Dashboard |
| `/register` | GET | Registrierungsseite |
| `/auth/register` | POST | Registrierungsformular, Weiterleitung zur Login-Seite |
| `/auth/logout` | POST | Session beenden |

## Middleware

| Schicht | Funktion |
|---|---|
| `TraceLayer` | Request/Response Logging |
| `Extension(PrometheusHandle)` | Metrics-Endpoint |
| `Extension(AppState)` | Datenbank und JWT-Konfiguration |
| `Extension(auth_state)` | JWT-Secret + Token-Expiry |
| `Extension(state)` | AppState (DB, app_name) |

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

## Dependencies

### common
- `tracing`
- `tracing-subscriber`
- `thiserror`
- `serde` (derive)
- `utoipa` (derive)

### config
- `clap` (derive, env)
- `dotenvy`

### server
- `tokio` (full)
- `axum`
- `topcoat`
- `metrics` + `metrics-exporter-prometheus`
- `anyhow`
- `serde_json`
- `utoipa`
- `utoipa-swagger-ui` (axum feature)
- `turso`
- `bcrypt`
- `jsonwebtoken` (JWT-Erzeugung)
- `common`, `config`, `web` (intern)
- `tower`, `tower-http` (trace)

### web
- `topcoat` (router, view features)
- `tokio`

## Usage

### Run
```bash
cargo install topcoat-cli
topcoat dev
```

`topcoat dev` führt das Build-Script aus, generiert das Tailwind-Stylesheet,
bündelt die Assets und startet anschließend den Server. Für einen manuellen
Start muss vorher ein Asset-Bundle mit `topcoat asset bundle` erzeugt werden.

### Run with CLI args
```bash
cargo run --package server -- --host 0.0.0.0 --port 3000 --log-level debug
```

### Environment
```bash
export HOST=0.0.0.0
export PORT=3000
export RUST_LOG=debug
export JWT_SECRET=mein-geheimes-secret
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
- Layout mit `#[layout("/")]`
- Das Frontend ist derzeit statisches SSR — keine `#[component]`, keine `$()`-Reaktivität
- Styling via Tailwind CSS, generated by Topcoat's Tailwind integration
- Form-Submits für Login/Register (kein SPA-Fetch)

### Middleware

| Schicht | Funktion |
|---|---|
| `TraceLayer` | Request/Response Logging |
| `Extension(PrometheusHandle)` | Metrics-Endpoint |
| `Extension(AppState)` | Datenbank und JWT-Konfiguration |
| `Extension(auth_state)` | JWT-Secret + Token-Expiry |
| `Extension(state)` | AppState (DB, app_name) |
