# Security & operations

## Metrics

`GET /api/v1/metrics` exposes Prometheus metrics:

- `tempoforge_http_requests_total`
- `tempoforge_http_request_duration_seconds`

Scrape with Prometheus; visualize in Grafana.

## API keys

- `GET /api/v1/keys`
- `POST /api/v1/keys` — returns `raw_key` once
- `POST /api/v1/keys/{id}/revoke`

Keys are stored as BLAKE3 hashes (`tf_live_…` prefix).

## Encrypted secrets

Org secrets (RPC URLs, private deploy keys metadata, etc.) are encrypted with `ENCRYPTION_KEY` before storage:

- `GET /api/v1/secrets`
- `POST /api/v1/secrets`
- `GET /api/v1/secrets/{name}` — audited reveal

## Hardening defaults

- Request body limit: 2 MiB
- Security headers: `nosniff`, `DENY` framing, strict referrer, `no-store`
- Rate limiting per client fingerprint
- Request IDs on every response
