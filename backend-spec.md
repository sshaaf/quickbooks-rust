# Vega Backend Specification

## Overview

Vega is a Rust backend service that integrates with the **QuickBooks Online Sandbox API** to pull financial data (customers, invoices, payments, accounts) and expose it via a REST API to an HTML/CSS frontend.

---

## Technology Stack

| Component         | Choice                                                                 |
| ----------------- | ---------------------------------------------------------------------- |
| Language          | Rust (stable)                                                          |
| Web Framework     | [Axum](https://docs.rs/axum)                                          |
| Async Runtime     | Tokio                                                                  |
| HTTP Client       | Reqwest (for outbound calls to QuickBooks)                             |
| Serialization     | Serde / serde_json                                                     |
| Configuration     | dotenvy (`.env` file)                                                  |
| Token Storage     | In-memory (single-user; upgradeable to SQLite/Postgres later)          |
| Logging           | tracing + tracing-subscriber                                          |
| CORS              | tower-http CorsLayer (to serve the HTML frontend from a different origin) |

---

## QuickBooks Sandbox Setup

### Prerequisites

1. Create an Intuit Developer account at <https://developer.intuit.com>.
2. Create an app and select the **com.intuit.quickbooks.accounting** scope.
3. Note the **Client ID**, **Client Secret**, and configure a **Redirect URI** (e.g., `http://localhost:3000/api/auth/callback`).
4. A sandbox company is automatically provisioned with sample data.

### Environment Variables (`.env`)

```
QBO_CLIENT_ID=<your_client_id>
QBO_CLIENT_SECRET=<your_client_secret>
QBO_REDIRECT_URI=http://localhost:3000/api/auth/callback
QBO_ENVIRONMENT=sandbox
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
```

---

## Authentication — OAuth 2.0 Authorization Code Flow

QuickBooks requires the standard OAuth 2.0 Authorization Code grant. The backend handles the full lifecycle.

### Endpoints

| Intuit URL                                                     | Purpose                          |
| -------------------------------------------------------------- | -------------------------------- |
| `https://appcenter.intuit.com/connect/oauth2`                  | Authorization (user consent)     |
| `https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer`   | Token exchange & refresh         |

### Token Lifecycle

- **Access token** expires in **1 hour**.
- **Refresh token** is valid for **100 days** (rolling).
- The backend must persist `realm_id`, `access_token`, `refresh_token`, and `expires_at` and refresh proactively before expiry.

### Auth Flow (sequence)

```
Browser                    Vega Backend                 Intuit
  |                            |                          |
  |-- GET /api/auth/login ---->|                          |
  |<-- 302 redirect ----------|-- builds auth URL ------->|
  |                            |                          |
  |   (user consents on Intuit)                           |
  |                            |<-- callback ?code&realm --|
  |                            |-- POST token exchange --->|
  |                            |<-- access + refresh token |
  |<-- 302 redirect to UI ----|                          |
  |                            |                          |
  |-- GET /api/customers ----->|                          |
  |                            |-- GET (bearer token) ---->|
  |                            |<-- JSON response ---------|
  |<-- JSON ------------------|                          |
```

---

## Backend REST API

All endpoints are prefixed with `/api`. The backend proxies and reshapes QuickBooks data for the frontend.

### Auth Routes

| Method | Path                   | Description                                    |
| ------ | ---------------------- | ---------------------------------------------- |
| GET    | `/api/auth/login`      | Redirects browser to Intuit OAuth consent page  |
| GET    | `/api/auth/callback`   | Handles OAuth callback, exchanges code for tokens |
| GET    | `/api/auth/status`     | Returns `{ connected: bool, company_name? }`   |
| POST   | `/api/auth/disconnect` | Clears stored tokens                           |

### Data Routes

| Method | Path                        | Description                             | QBO Entity  |
| ------ | --------------------------- | --------------------------------------- | ----------- |
| GET    | `/api/customers`            | List all customers                      | Customer    |
| GET    | `/api/customers/:id`        | Get a single customer                   | Customer    |
| GET    | `/api/invoices`             | List all invoices                       | Invoice     |
| GET    | `/api/invoices/:id`         | Get a single invoice                    | Invoice     |
| GET    | `/api/payments`             | List all payments                       | Payment     |
| GET    | `/api/accounts`             | List all accounts (Chart of Accounts)   | Account     |
| GET    | `/api/company`              | Get company info                        | CompanyInfo |
| GET    | `/api/profit-and-loss`      | Profit & Loss report                    | Report      |
| GET    | `/api/balance-sheet`        | Balance Sheet report                    | Report      |

### Query Parameters (data routes)

| Param       | Type   | Description                                      |
| ----------- | ------ | ------------------------------------------------ |
| `page`      | u32    | Page number (default 1)                          |
| `per_page`  | u32    | Items per page (default 25, max 100)             |
| `start_date`| string | Filter start date (YYYY-MM-DD), for reports      |
| `end_date`  | string | Filter end date (YYYY-MM-DD), for reports        |

### Standard Response Envelope

```json
{
  "data": [ ... ],
  "meta": {
    "page": 1,
    "per_page": 25,
    "total_count": 142
  }
}
```

Error responses:

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "QuickBooks connection expired. Please reconnect."
  }
}
```

---

## QuickBooks API Integration Layer

### Base URL (Sandbox)

```
https://sandbox-quickbooks.api.intuit.com/v3/company/{realmId}/
```

### Query Language

QuickBooks uses a SQL-like query syntax passed as a URL-encoded query parameter:

```
GET /v3/company/{realmId}/query?query=SELECT * FROM Customer STARTPOSITION 1 MAXRESULTS 25
```

All requests require the header:

```
Authorization: Bearer <access_token>
Accept: application/json
```

### Automatic Token Refresh

Before every outbound QBO request, the backend checks whether the access token is within 5 minutes of expiry. If so, it performs a refresh using the refresh token endpoint before proceeding.

---

## Project Structure

```
vega/
├── backend-spec.md
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs              # Server bootstrap, router setup
│   ├── config.rs            # Env var loading and app config struct
│   ├── error.rs             # Unified error type and API error responses
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── handlers.rs      # /api/auth/* route handlers
│   │   ├── oauth.rs         # OAuth2 URL building, token exchange, refresh
│   │   └── store.rs         # In-memory token store (Arc<RwLock<...>>)
│   ├── qbo/
│   │   ├── mod.rs
│   │   ├── client.rs        # QBO HTTP client with auto-refresh
│   │   ├── types.rs         # QBO response structs (Customer, Invoice, etc.)
│   │   └── queries.rs       # Query builders for each entity
│   ├── api/
│   │   ├── mod.rs
│   │   ├── customers.rs     # /api/customers handlers
│   │   ├── invoices.rs      # /api/invoices handlers
│   │   ├── payments.rs      # /api/payments handlers
│   │   ├── accounts.rs      # /api/accounts handlers
│   │   ├── company.rs       # /api/company handler
│   │   └── reports.rs       # /api/profit-and-loss, /api/balance-sheet
│   └── models.rs            # Shared response models (envelope, pagination)
└── frontend/                # HTML/CSS frontend (separate phase)
    ├── index.html
    ├── css/
    └── js/
```

---

## Key Cargo Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"
tower-http = { version = "0.6", features = ["cors", "fs"] }
tracing = "0.1"
tracing-subscriber = "0.3"
base64 = "0.22"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

---

## Security Considerations

1. **Client Secret** — Never exposed to the frontend; all OAuth operations happen server-side.
2. **CSRF Protection** — The OAuth `state` parameter is a random UUID verified on callback.
3. **Token Storage** — Tokens are held in-memory behind `Arc<RwLock<>>`. For production, migrate to encrypted DB storage.
4. **CORS** — Restricted to `http://localhost:*` during development.
5. **HTTPS** — Not enforced locally, but required for any non-sandbox deployment.

---

## Development Workflow

```bash
# 1. Copy env template and fill in credentials
cp .env.example .env

# 2. Run the backend
cargo run

# 3. Open browser to initiate QuickBooks connection
#    http://localhost:3000/api/auth/login

# 4. After OAuth callback, data endpoints become available
#    http://localhost:3000/api/customers
```

---

## Future Enhancements (out of scope for v1)

- Persistent token storage (SQLite / Postgres)
- Webhook support for real-time QuickBooks change notifications
- Write-back operations (create invoices, record payments)
- Multi-company / multi-user support
- Rate limiting and request queuing (QBO enforces 500 req/min)
- Background sync / caching layer to reduce QBO round-trips
