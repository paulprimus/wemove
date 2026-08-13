# Authentifizierungsablauf

## Übersicht

Das System implementiert einen OAuth 2.1-kompatiblen Auth-Server mit drei Flows:

1. **User Login** — Email/Password-Auth mit DB-Abfrage und JWT-Token
2. **User Registration** — Neuen User anlegen mit bcrypt-Passwort-Hashing
3. **Client Credentials Flow** — Token-Generierung ohne User-Auth
4. **Authorization Code Flow mit PKCE** — sicherer Flow für Benutzer-Auth

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          WeMove Server                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      routes.rs                                    │  │
│  │  POST /api/auth/login      →  auth_rest.rs (User-Login)          │  │
│  │  POST /api/auth/token      →  auth_rest.rs (Client Credentials)  │  │
│  │  POST /api/auth/register   →  auth_rest.rs (User-Registrierung)  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                      │                                   │
│                                      ▼                                   │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      marvels_auth (Library)                       │  │
│  │  AppState::create_access_token() — JWT-Erstellung (HS256)        │  │
│  │  verify_pkce() — PKCE-Verifikation (timing-sicher)               │  │
│  │  DTOs: JsonTokenRequest, JsonErrorResponse                        │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                      │                                   │
│                                      ▼                                   │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      SQLite (Turso)                               │  │
│  │  users: id, email, name, password_hash, created_at                 │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Flow 1: User Login

```
Client                    Server                          SQLite (Turso)
  │                          │                                    │
  │──POST /api/auth/login───▶│                                    │
  │  email, password (Form)  │                                    │
  │                          │──SELECT ... WHERE email=?─────────▶│
  │                          │◀─User{...password_hash}───────────│
  │                          │──bcrypt::verify()─────────────────▶│
  │                          │                                    │
  │                          │──JWT (create_access_token)─────────│
  │◀─{success, token}────────│                                    │
```

**Request** (`crates/server/src/auth_rest.rs:138`):
```
POST /api/auth/login
Content-Type: application/x-www-form-urlencoded

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

**Verarbeitung** (`crates/server/src/auth_rest.rs:138` + `crates/server/src/user_repo.rs`):
1. `UserRepository::find_by_email(email)` → lädt User mit `password_hash` aus DB
2. `UserRepository::verify_password()` → `bcrypt::verify(password, hash)` → `Result<bool, ...>`
3. Bei Erfolg: `marvels_auth::AppState::create_access_token(email, "read write")` → JWT

**Fehler:**
| Fall | Status | Message |
|---|---|---|
| Email nicht gefunden | 401 | "Invalid email or password" |
| Falsches Passwort | 401 | "Invalid email or password" |
| DB-Fehler | 500 | "Database error" |

## Flow 2: User Registration

```
Client                    Server                          SQLite (Turso)
  │                          │                                    │
  │──POST /auth/register────▶│                                    │
  │  name, email, password   │                                    │
  │  (Form)                   │                                    │
  │                          │──bcrypt::hash(password, DEFAULT)───▶│
  │                          │──INSERT INTO users ...─────────────▶│
  │◀─{success, user_id}──────│                                    │
```

**Request** (`crates/server/src/auth_rest.rs:209`):
```
POST /auth/register
Content-Type: application/x-www-form-urlencoded

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

**Verarbeitung** (`crates/server/src/auth_rest.rs:209` + `crates/server/src/user_repo.rs:90`):
1. Validierung: name, email, password dürfen nicht leer sein
2. `bcrypt::hash(password, DEFAULT_COST)` → `password_hash` (bcrypt cost = 12)
3. `UserRepository::create(CreateUser{...})` → `INSERT INTO users`
4. UNIQUE-Constraint auf `email` → `AppError::Conflict`

## Flow 3: Client Credentials

```
Client                    Server                          marvels_auth
  │                          │                                  │
  │──POST /api/auth/token───▶│                                  │
  │  {client_id, scope?}     │                                  │
  │  (JSON)                   │                                  │
  │                          │──JWT (create_access_token)────────│
  │◀─{access_token, ...}────│                                  │
```

**Verwendungszweck**: Direkte Authentifizierung mit Client-ID, kein User-Auth.

**Request** (`crates/server/src/auth_rest.rs:36`):
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

> **Sicherheitshinweis**: Der `client_secret`-Wert aus der Request wird akzeptiert
> aber aktuell **nicht validiert**. Dies ist eine bekannte Lücke.

## Flow 4: PKCE-Funktionalität (Bibliothek, keine HTTP-Routen)

Der Authorization Code + PKCE Flow ist **nicht** als HTTP-Routen implementiert.
Die `marvels_auth`-Bibliothek stellt jedoch die Bausteine bereit, die bei Bedarf
als Routen im server-Crate zusammengeführt werden können:

### `verify_pkce()`

**Implementierung** (`marvels_auth::authentication::verify_pkce`):

```rust
pub fn verify_pkce(code_verifier: &str, code_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let computed = URL_SAFE_NO_PAD.encode(hash);
    constant_time_eq(&computed, code_challenge)
}
```

- Verwendet SHA-256 Hash
- Base64URL-Encoding ohne Padding
- Timing-sicherer Vergleich (verhindert Timing-Angriffe)

### `AppState::create_access_token()`

Generiert ein JWT (HS256) mit Claims: `sub`, `scope`, `iat`, `exp`. Kann für
jeden Auth-Flow verwendet werden.

## JWT Token Struktur

**Claims** (`crates/server/src/auth_rest.rs:17-22`):
```json
{
  "sub": "client_id oder email",
  "scope": "read write",
  "exp": 1234567890,
  "iat": 1234567890
}
```

**Algorithmus**: HS256 (HMAC mit SHA-256) via `jsonwebtoken`-Crate in `marvels_auth`

## PKCE Verifikation

**Implementierung** (`../marvels/marvels_auth/src/authentication.rs`):

```rust
pub fn verify_pkce(code_verifier: &str, code_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let computed = URL_SAFE_NO_PAD.encode(hash);
    constant_time_eq(&computed, code_challenge)
}
```

- Verwendet SHA-256 Hash
- Base64URL-Encoding ohne Padding
- Timing-sicherer Vergleich (verhindert Timing-Angriffe)

## Endpunkte Übersicht

Alle HTTP-Routen werden im server-Crate definiert. marvels_auth exponiert keine eigenen Routen.

| Pfad | Methode | Input | Beschreibung |
|------|---------|-------|--------------|
| `/api/auth/login` | POST | Form | User-Login mit Email/Passwort |
| `/api/auth/token` | POST | JSON | Client Credentials Token |
| `/api/auth/register` | POST | Form | User-Registrierung |

## Fehlerbehandlung

### Framework-agnostisch (`common/src/error.rs`)

```rust
pub enum AppError {
    Internal(String),      // 500 + logging
    BadRequest(String),    // 400
    NotFound(String),      // 404
    Unauthorized(String),  // 401
    Conflict(String),      // 409
    Database(#[from] DbError), // 500 + logging
}

pub enum DbError {
    Connection(String),
    Query(String),
    Constraint(String),
    PasswordHash(String),
}
```

### HTTP-Mapping (`crates/server/src/error.rs`)

| AppError | HTTP Status | Logging |
|---|---|---|
| `Internal` | 500 | ✅ |
| `BadRequest` | 400 | ❌ |
| `NotFound` | 404 | ❌ |
| `Unauthorized` | 401 | ❌ |
| `Conflict` | 409 | ❌ |
| `Database` | 500 | ✅ |

### Handler-spezifisch (`crates/server/src/auth_rest.rs`)
Einige Handler bauen Responses manuell (direktes `StatusCode::UNAUTHORIZED` etc.), weil
sie verschiedene Response-Typen (`LoginResponse`, `RegisterResponse`) zurückgeben müssen.

## Sicherheitsmerkmale

1. **PKCE**: Schutz vor Authorization Code Injection
2. **Timing-sicherer Vergleich**: Verhindert Timing-Angriffe in `verify_pkce`
3. **Einmalige Auth-Codes**: Wiederverwendung nicht möglich (aus Store entfernt)
4. **JWT mit Ablaufzeit**: Token laufen automatisch ab
5. **Bcrypt-Passwort-Hashing**: DEFAULT_COST = 12
6. **In-Memory Store**: Auth-Codes nicht persistiert (Single-Node)

## Offene Sicherheitslücken

1. **Client Secret wird nicht geprüft** im `/api/auth/token`-Endpoint
2. **`code_challenge_method`-Validierung fehlt** in `marvels_auth::server::authenticate`
3. **Kein Login-State im Frontend**: JWT wird als JSON zurückgegeben, aber nicht
   client-seitig gespeichert oder genutzt
4. **marvels_auth** ist ein externer Workspace — `JWT_SECRET` muss als ENV gesetzt werden
   (kein Fallback), härtes `expect()` bei fehlender Konfiguration