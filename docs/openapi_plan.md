# OpenAPI-Integration für WeMove (Rust/Axum + Topcoat)

## Status

Die Backend-OpenAPI-Integration (Schritte 1-5) ist umgesetzt. Die automatische
Client-Generierung für das Topcoat-Frontend (Schritte 6-9) ist **offen**.

## Problem

Die API-Endpoints werden zur Compilezeit als OpenAPI-Spec generiert (utoipa), aber:
1. Nicht alle Handler sind in `ApiDoc` registriert (login, register, health fehlen)
2. Auth-DTOs (`LoginRequest`, `RegisterRequest`, etc.) fehlen in den OpenAPI-Schemas
3. Das Topcoat-Frontend nutzt keine generierten API-Typen

## Umsetzung (abgeschlossen)

### Schritt 1-2: Dependencies + ToSchema-Annotation ✅

**`crates/common/src/lib.rs`**: Alle DTOs mit `#[derive(ToSchema, Serialize, Deserialize)]`:
- `HealthResponse`, `MainRequest`, `MainResponse`
- `LoginRequest`, `LoginResponse`, `RegisterRequest`, `RegisterResponse`

### Schritt 3-4: Handler-Annotation + ApiDoc ✅

**`crates/server/src/handlers.rs`**:
```rust
#[utoipa::path(get, path = "/api/main", tag = "main", responses(...))]
pub async fn main_get() -> Json<MainResponse> { ... }

#[utoipa::path(post, path = "/api/main", tag = "main", ...)]
pub async fn main_post(...) -> Result<Json<MainResponse>, ApiError> { ... }

#[utoipa::path(get, path = "/api/health", tag = "health", ...)]
pub async fn health() -> Json<HealthResponse> { ... }
```

**`crates/server/src/openapi.rs`**: `ApiDoc` sammelt:
- Pfade: `main_get`, `main_post`, `health`, `token`
- Schemas: `MainRequest`, `MainResponse`, `HealthResponse`, `JsonTokenRequest`, `TokenResponse`, `JsonErrorResponse`

### Schritt 5: Swagger-UI ✅

**`crates/server/src/routes.rs:57`**:
```rust
.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
```

## Offene Punkte

### 1. Nicht alle Handler in ApiDoc

Folgende Handler sind **nicht** in `ApiDoc` registriert:
- `auth_rest::login` — `#[utoipa::path]` fehlt
- `auth_rest::register` — `#[utoipa::path]` fehlt
- `handlers::health` — ist registriert ✅

Folgende Schemas fehlen in `ApiDoc`:
- `LoginRequest`, `LoginResponse`
- `RegisterRequest`, `RegisterResponse`

**Empfehlung**: `ApiDoc` in `openapi.rs` erweitern:
```rust
paths(
    super::handlers::main_get,
    super::handlers::main_post,
    super::handlers::health,
    super::auth_rest::token,
    super::auth_rest::login,      // hinzufügen
    super::auth_rest::register,   // hinzufügen
),
components(schemas(
    // ...existing...
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
))
```

### 2. Kein Frontend-API-Client

Das Topcoat-Frontend (`crates/web`) nutzt **keine generierten API-Typen**.
Auth-Endpoints werden per HTML-Form-Submit aufgerufen (kein SPA-Fetch).
Der JWT-Token wird als JSON zurückgegeben, aber client-seitig nicht gespeichert.

## Todos

| # | ID | Titel | Beschreibung | Status |
|---|----|----|----|----|
| 1 | `add-utoipa-deps` | Adding utoipa dependencies | Workspace + server + common | ✅ Abgeschlossen |
| 2 | `annotate-dtos` | Annotating DTOs with ToSchema | MainRequest/Response, HealthResponse, Login*, Register* | ✅ Abgeschlossen |
| 3 | `annotate-handlers` | Annotating handlers with utoipa::path | main_get, main_post, health | ✅ Abgeschlossen |
| 4 | `create-apidoc` | Creating ApiDoc module | `openapi.rs` mit ApiDoc | ✅ Abgeschlossen |
| 5 | `mount-swagger-ui` | Mounting Swagger UI route | `/swagger-ui`, `/api-docs/openapi.json` | ✅ Abgeschlossen |
| 6 | `extend-apidoc` | Extending ApiDoc with all handlers | login, register, LoginRequest/Response, RegisterRequest/Response | ❌ Offen |
| 7 | `annotate-auth-handlers` | Annotating auth handlers | `#[utoipa::path]` auf login und register | ❌ Offen |
| 8 | `verify-openapi` | Verifying OpenAPI spec | Server starten, `/api-docs/openapi.json` prüfen | ❌ Offen |

## Hinweise

- Spec-Generierung erfolgt zur Compilezeit aus dem Code — keine manuelle YAML/JSON-Pflege
- Single Source of Truth: Rust-Code mit utoipa-Makros
- `web`-Crate ist Topcoat (Rust-SSR), **kein** Angular/TypeScript — ein TS-Client-Generator ist daher nicht relevant
- Generierte API-Typen für Topcoat wären nur relevant, wenn das Frontend SPA-style Fetch-Aufrufe macht (aktuell: Form-Submits)