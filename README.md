# Vega

Rust backend + server-rendered UI for viewing QuickBooks Online data. Built with Axum, Askama, HTMX, and Tailwind CSS.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- An [Intuit Developer](https://developer.intuit.com) account with a QuickBooks app

## Setup

1. **Clone and configure:**

```bash
cp .env.example .env
```

Edit `.env` with your Intuit app credentials:

```
QBO_CLIENT_ID=your_client_id
QBO_CLIENT_SECRET=your_client_secret
QBO_REDIRECT_URI=http://localhost:3000/api/auth/callback
QBO_ENVIRONMENT=sandbox
```

2. **Register the redirect URI** in the Intuit Developer portal under your app's **Settings > Redirect URIs > Development**. Add exactly:

```
http://localhost:3000/api/auth/callback
```

3. **Run:**

```bash
cargo run
```

4. **Connect:** Open [http://localhost:3000](http://localhost:3000) and click **Connect to QuickBooks**. Authorize the sandbox company when prompted.

## Routes

### UI (HTML)

| Path | Description |
|---|---|
| `/` | Dashboard |
| `/customers` | Customer list (click row for detail) |
| `/invoices` | Invoice list (click row for line items) |
| `/payments` | Payment list |
| `/accounts` | Chart of accounts |
| `/company` | Company info |
| `/reports/profit-and-loss` | Profit & Loss report |
| `/reports/balance-sheet` | Balance Sheet report |

### API (JSON)

All UI routes have a JSON counterpart under `/api/` (e.g., `/api/customers`, `/api/invoices/:id`). Pagination via `?page=1&per_page=25`. Reports accept `?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD`.

## Project Structure

```
src/
├── main.rs          # Server bootstrap and routing
├── config.rs        # Environment config
├── error.rs         # Unified error handling
├── models.rs        # Shared request/response types
├── auth/            # OAuth 2.0 flow (login, callback, token store)
├── qbo/             # QuickBooks API client, types, query builder
├── api/             # JSON endpoint handlers
└── views/           # HTML view handlers + view models
templates/           # Askama (Jinja2-style) HTML templates
```

## Notes

- Tokens are stored in-memory — reconnect after server restart.
- Access tokens auto-refresh before expiry (1-hour lifetime).
- Sandbox comes pre-populated with sample data from Intuit.
