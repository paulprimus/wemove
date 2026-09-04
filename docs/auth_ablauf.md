# Authentifizierungsablauf

## Übersicht

Das System implementiert drei Authentifizierungs-Flows:

1. **User Login** — Email/Password-Auth mit DB-Abfrage und JWT-Token
2. **User Registration** — Neuen User anlegen mit bcrypt-Passwort-Hashing
3. **Client Credentials Flow** — Token-Generierung ohne User-Auth
4. **Web-Session** — Login über Topcoat-Formular mit serverseitiger Session

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
│  │                  JWT / Passwortverarbeitung                       │  │
│  │  AppState::create_access_token() — JWT-Erstellung (HS256)        │  │
│  │  jsonwebtoken + bcrypt + DTOs aus common                           │  │
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
3. Bei Erfolg: `AppState::create_access_token(email, "read write")` → JWT

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
  │──POST /api/auth/register─▶│                                    │
  │  name, email, password   │                                    │
  │  (Form)                   │                                    │
  │                          │──bcrypt::hash(password, DEFAULT)───▶│
  │                          │──INSERT INTO users ...─────────────▶│
  │◀─{success, user_id}──────│                                    │
```

**Request** (`crates/server/src/auth_rest.rs:209`):
```
POST /api/auth/register
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
Client                    Server                          JWT-Erzeugung
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

> **Sicherheitshinweis**: Der optionale `client_secret`-Wert wird zwar eingelesen,
> aber aktuell nicht validiert. Dies ist eine bekannte Lücke.

## Flow 4: Web-Session

Die Topcoat-Webrouten verwenden zusätzlich Cookie- und Session-Unterstützung.
Nach erfolgreichem Login wird eine Session gestartet und der Benutzer zum
Dashboard weitergeleitet. Beim Logout wird die Session beendet und der
zugehörige Eintrag aus der Datenbank entfernt.

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

**Algorithmus**: HS256 (HMAC mit SHA-256) via `jsonwebtoken`-Crate.

## Endpunkte Übersicht

Alle HTTP-Routen werden im `server`-Crate definiert.

| Pfad | Methode | Input | Beschreibung |
|------|---------|-------|--------------|
| `/api/auth/login` | POST | Form | User-Login mit Email/Passwort |
| `/api/auth/token` | POST | JSON | Client Credentials Token |
| `/api/auth/register` | POST | Form | User-Registrierung |
| `/auth/login` | POST | Form | Web-Login mit Session |
| `/auth/register` | POST | Form | Web-Registrierung mit Redirect |
| `/auth/logout` | POST | - | Web-Session beenden |

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

1. **JWT mit Ablaufzeit**: Token laufen automatisch ab
2. **Bcrypt-Passwort-Hashing**: `DEFAULT_COST = 12`
3. **Session-Cookies** für den serverseitigen Web-Login

## Offene Sicherheitslücken

1. **Client Secret wird nicht geprüft** im `/api/auth/token`-Endpoint
2. **Kein Login-State im REST-Frontend**: JWT wird als JSON zurückgegeben, aber nicht
   client-seitig gespeichert oder genutzt
3. **`JWT_SECRET` muss als ENV gesetzt werden**; bei fehlender Konfiguration beendet
    sich der Server absichtlich mit `expect()`.
