# Authentifizierungsablauf

## Übersicht

Das System implementiert einen OAuth 2.1-kompatiblen Auth-Server mit zwei möglichen Flows:

1. **Client Credentials Flow** - für direkte Service-zu-Service-Kommunikation
2. **Authorization Code Flow mit PKCE** - für Benutzer-/Client-Authentifizierung

## Architektur

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          WeMove Server                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      routes.rs                                    │  │
│  │  POST /api/auth/token  →  auth_rest.rs (marvels_auth::rest)     │  │
│  │  /auth/*              →  marvels_auth Router                    │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                      │                                   │
│                                      ▼                                   │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      marvels_auth (Library)                       │  │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐         │  │
│  │  │ /authenticate │  │  /authorize   │  │  /protected   │         │  │
│  │  └───────────────┘  └───────────────┘  └───────────────┘         │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Flow 1: Client Credentials (einfacher)

```
Client                    Server                          marvels_auth
  │                          │                                  │
  │──POST /api/auth/token───▶│                                  │
  │  {client_id, client_secret}                                  │
  │                          │──JSON /token────────────────────▶│
  │                          │  {grant_type: "client_credentials"} │
  │                          │                                  │
  │                          │◀──JWT Access Token──────────────│
  │◀─{access_token, ...}────│                                  │
```

**Verwendungszweck**: Direkte Authentifizierung mit Client-ID/Secret, kein User-Auth.

**Request** (`crates/server/src/auth_rest.rs:22-26`):
```json
{
  "client_id": "mein-client",
  "client_secret": "geheim",
  "scope": "read write"
}
```

**Response**:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read write"
}
```

## Flow 2: Authorization Code mit PKCE (sicherer)

```
Client                    Server                          marvels_auth
  │                          │                                  │
  │                          │◀─POST /authenticate─────────────│
  │                          │  {client_id, code_challenge}    │
  │                          │                                  │
  │                          │◀─Auth-Code (UUID)───────────────│
  │◀─Auth-Code───────────────│                                  │
  │                          │                                  │
  │                          │◀─POST /authorize────────────────│
  │                          │  {code, code_verifier}          │
  │                          │                                  │
  │                          │◀─JWT Access Token───────────────│
  │◀─{access_token, ...}─────│                                  │
```

### Schritt 1: Authentifizieren (`/authenticate`)

**Endpoint**: `POST /auth/authenticate` (Protobuf)

**Request** (Protobuf):
- `client_id`: Client-Identifier
- `code_challenge`: BASE64URL(SHA256(code_verifier))
- `code_challenge_method`: muss "S256" sein

**Verarbeitung** (`marvels_auth` crate):
1. Validiert `code_challenge_method` (nur S256 erlaubt)
2. Generiert UUID als Authorization Code
3. Speichert in-memory: `{auth_code → AuthCodeEntry{client_id, code_challenge}}`

**Response**:
- `subject`: Authorization Code (UUID)

### Schritt 2: Autorisieren (`/authorize`)

**Endpoint**: `POST /auth/authorize` (Protobuf)

**Request**:
- `grant_type`: "authorization_code"
- `code`: Authorization Code aus Schritt 1
- `code_verifier`: PKCE Verifier

**Verarbeitung** (`marvels_auth` crate):
1. Entfernt Auth-Code aus Store (einmalige Verwendung)
2. Verifiziert PKCE: `BASE64URL(SHA256(code_verifier)) == code_challenge`
3. Generiert JWT Access Token

### Schritt 3: Geschützte Ressource (`/protected`)

**Endpoint**: `GET /auth/protected`

**Header**: `Authorization: Bearer <access_token>`

**Verarbeitung** (`marvels_auth` crate):
1. Extrahiert Bearer Token aus Header
2. Validiert JWT-Signatur mit HS256
3. Gibt geschützte Ressource zurück

## JWT Token Struktur

**Claims** (`marvels_auth` crate):
```json
{
  "sub": "client_id",
  "scope": "read write",
  "exp": 1234567890,
  "iat": 1234567890
}
```

**Algorithmus**: HS256 (HMAC mit SHA-256)

## PKCE Verifikation

**Implementierung** (`marvels_auth` crate):

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

| Pfad | Methode | Protokoll | Beschreibung |
|------|---------|-----------|--------------|
| `/api/auth/token` | POST | JSON | Client Credentials (via marvels_auth) |
| `/auth/authenticate` | POST | Protobuf | Auth-Code anfordern |
| `/auth/authorize` | POST | Protobuf | Token austauschen |
| `/auth/protected` | GET | - | Geschützte Ressource |

## Fehlerbehandlung

| Fehler | HTTP Status | Beschreibung |
|--------|-------------|--------------|
| `invalid_request` | 400 | Ungültige PKCE-Methode |
| `invalid_grant` | 401 | Ungültiger Auth-Code oder PKCE-Fehler |
| `unsupported_grant_type` | 400 | Unbekannter Grant Type |
| `server_error` | 502 | Interne Fehler |

## Sicherheitsmerkmale

1. **PKCE**: Schutz vor Authorization Code Injection
2. **Timing-sicherer Vergleich**: Verhindert Timing-Angriffe
3. **Einmalige Auth-Codes**: Wiederverwendung nicht möglich
4. **JWT mit Ablaufzeit**: Token laufen automatisch ab
5. **In-Memory Store**: Auth-Codes nicht persistiert (Single-Node)