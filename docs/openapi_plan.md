# OpenAPI-Integration für WeMove (Rust/Axum + Angular)

## Problem
Aktuell gibt es keine maschinenlesbare API-Beschreibung. Endpoints (`/`, `/health`, `/metrics`)
werden nur manuell in `docs/SPEC.md` dokumentiert. Das Angular-Frontend (`web/`) hat keine
generierten Typen/Clients für die Rust-DTOs (`HelloWorldRequest`, `HelloWorldResponse`,
`HealthResponse` in `common`).

## Ansatz
- **utoipa** als OpenAPI-Generator für Axum einführen (Makro-basiert, sehr verbreitet).
- **utoipa-swagger-ui** für eine interaktive Swagger-UI unter `/swagger-ui` und
  JSON-Spec unter `/api-docs/openapi.json`.
- DTOs in `crates/common` mit `#[derive(ToSchema)]` annotieren, Handler in
  `crates/server/src/handlers.rs` mit `#[utoipa::path(...)]` versehen.
- Zentrales `ApiDoc`-Struct (`#[derive(OpenApi)]`) in `server`, das Pfade + Schemas sammelt.
- **TypeScript-Client-Generierung** für `web/`: `openapi-typescript` (oder
  `openapi-typescript-codegen`) als npm-Devdependency, das aus der laufenden/JSON-Spec
  TS-Typen/Client generiert (npm-Script `generate:api`).
- `docs/SPEC.md` um Hinweis auf Swagger-UI/OpenAPI-Workflow ergänzen.

## Betroffene Dateien
- `crates/server/Cargo.toml` – neue Dependencies `utoipa`, `utoipa-swagger-ui`, `utoipa-axum` (optional)
- `crates/common/Cargo.toml` + DTOs (`ToSchema`-Derive, evtl. Cargo-Feature-Flag um utoipa nicht zwingend überall zu brauchen)
- `crates/server/src/handlers.rs` – `#[utoipa::path]`-Annotationen je Handler
- `crates/server/src/routes.rs` – Swagger-UI-Route einhängen, `ApiDoc` mounten
- neue Datei `crates/server/src/openapi.rs` – `ApiDoc`-Definition
- `web/package.json` – Devdependency + Script für Client-Generierung
- neue generierte Datei(en) unter `web/src/app/api/` (generierter Client, .gitignore-Frage klären)
- `docs/SPEC.md` – Abschnitt zu OpenAPI/Swagger ergänzen

## Todos

| # | ID | Titel | Beschreibung | Abhängig von |
|---|----|----|----|----|
| 1 | `add-utoipa-deps` | Adding utoipa dependencies | `utoipa`, `utoipa-swagger-ui` (und optional `utoipa-axum`) zu `Cargo.toml` (Workspace) und `crates/server/Cargo.toml` hinzufügen. `utoipa` (mit `derive`-Feature) zu `crates/common/Cargo.toml` hinzufügen, damit DTOs `ToSchema` ableiten können. | – |
| 2 | `annotate-dtos` | Annotating DTOs with ToSchema | `#[derive(ToSchema)]` zu `HelloWorldRequest`, `HelloWorldResponse`, `HealthResponse` in `crates/common` hinzufügen (neben bestehendem `Serialize`/`Deserialize`). | 1 |
| 3 | `annotate-handlers` | Annotating handlers with utoipa::path | `#[utoipa::path(...)]`-Makros zu `hello_world`, `hello_world_post`, `health` in `crates/server/src/handlers.rs` hinzufügen (Methode, Pfad, Request-/Response-Bodies, Statuscodes). | 1 |
| 4 | `create-apidoc` | Creating ApiDoc module | Neue Datei `crates/server/src/openapi.rs` mit `#[derive(OpenApi)] struct ApiDoc`, die alle Pfade und Schemas sammelt. | 2, 3 |
| 5 | `mount-swagger-ui` | Mounting Swagger UI route | `crates/server/src/routes.rs` anpassen: `SwaggerUi` (utoipa-swagger-ui) in den Axum-Router mergen, ApiDoc-JSON unter `/api-docs/openapi.json`, interaktive UI unter `/swagger-ui`. | 4 |
| 6 | `verify-backend-build` | Verifying backend builds and serves spec | `cargo build --workspace` und `cargo test --workspace` ausführen, Server starten und `/api-docs/openapi.json` sowie `/swagger-ui` per curl prüfen. | 5 |
| 7 | `add-ts-client-gen` | Adding TS client generation to Angular app | `openapi-typescript` (oder `openapi-typescript-codegen`) als devDependency in `web/package.json` hinzufügen, npm-Script `generate:api` ergänzen, das aus der Spec TS-Typen/Client nach `web/src/app/api/` generiert. Generiertes Ergebnis in `.gitignore` aufnehmen. | 6 |
| 8 | `verify-ts-client-gen` | Verifying TS client generation works | `npm run generate:api` in `web/` gegen laufendes Backend (oder exportierte Spec-Datei) ausführen und prüfen, dass generierte Typen mit `ng build`/`tsc` kompilieren. | 7 |
| 9 | `update-docs` | Updating SPEC.md documentation | Abschnitt in `docs/SPEC.md` zu OpenAPI/Swagger-UI-Setup und `generate:api`-Workflow ergänzen, Endpoints-/Dependencies-Abschnitte aktualisieren. | 8 |

## Offene Punkte / Hinweise
- Generierter TS-Client: entweder eingecheckt oder als Build-Step (`npm run generate:api`)
  vor `ng build`/`ng serve` – Vorschlag: Build-Step, nicht eingecheckt (via `.gitignore`),
  um Drift zu vermeiden.
- Spec-Generierung erfolgt zur Compile-/Laufzeit aus dem Code (kein separates Schreiben
  von YAML/JSON nötig) – Single Source of Truth bleibt Rust-Code.
