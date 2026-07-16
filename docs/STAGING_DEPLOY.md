# Staging deploy pack

This guide gets TempoForge AI onto a private staging environment with production auth hard-fail and locked CORS.

## What changed for deploy safety

- `APP_ENV=staging|production` **refuses to boot** without Clerk JWKS/issuer, a real `ENCRYPTION_KEY`, and a non-local `APP_URL`
- `Bearer dev` is **disabled** outside development
- CORS is restricted to `CORS_ORIGINS` / `APP_URL` (no `*`)
- Staging Docker Compose + API/Web Dockerfiles
- Fly.io and Railway starter configs
- `scripts/deploy-check.sh` preflight

## 1) Fill secrets

```bash
cp .env.staging.example .env.staging
# edit .env.staging — every value that says change_me / replace / pk_test_
openssl rand -hex 32   # use as ENCRYPTION_KEY
bash scripts/deploy-check.sh .env.staging
```

## 2) Local staging stack (Docker)

Requires Docker Desktop running.

```bash
docker compose \
  -f infrastructure/docker/docker-compose.staging.yml \
  --env-file .env.staging \
  up -d --build
```

Smoke tests:

```bash
curl -fsS "$NEXT_PUBLIC_API_URL/api/v1/health"
curl -fsS "$APP_URL" >/dev/null
# Authed request without Clerk JWT must fail:
curl -i -H 'Authorization: Bearer dev' "$NEXT_PUBLIC_API_URL/api/v1/projects"
# expect 401
```

## 3) Fly.io

```bash
fly apps create tempoforge-api
fly apps create tempoforge-web

# Attach a managed Postgres (or set DATABASE_URL to Neon/Supabase)
fly postgres create --name tempoforge-db
fly postgres attach tempoforge-db -a tempoforge-api

fly secrets set -a tempoforge-api \
  APP_ENV=staging \
  ALLOW_DEV_AUTH=false \
  APP_URL=https://tempoforge-web.fly.dev \
  CORS_ORIGINS=https://tempoforge-web.fly.dev \
  CLERK_JWKS_URL=... \
  CLERK_ISSUER=... \
  ENCRYPTION_KEY=$(openssl rand -hex 32) \
  GROQ_API_KEY=... \
  STRIPE_SECRET_KEY=... \
  STRIPE_WEBHOOK_SECRET=...

fly deploy -c infrastructure/fly/fly.api.toml

fly secrets set -a tempoforge-web CLERK_SECRET_KEY=...
fly deploy -c infrastructure/fly/fly.web.toml \
  --build-arg NEXT_PUBLIC_API_URL=https://tempoforge-api.fly.dev \
  --build-arg NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=pk_test_...
```

## 4) Railway

1. New project → deploy from GitHub repo `tempo_forge_ai`
2. API service: Dockerfile `infrastructure/docker/Dockerfile.api` (see `infrastructure/railway/railway.toml`)
3. Web service: Dockerfile `infrastructure/docker/Dockerfile.web` with build args for `NEXT_PUBLIC_*`
4. Add managed Postgres + Redis plugins; copy vars from `.env.staging.example`
5. Set `APP_ENV=staging` and `ALLOW_DEV_AUTH=false`

## 5) Stripe webhook (staging)

Endpoint:

```text
POST {API_PUBLIC_URL}/api/v1/billing/webhook
```

Events:

- `checkout.session.completed`
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`

## 6) Production auth hard-fail checklist

- [ ] `APP_ENV=staging` or `production`
- [ ] `ALLOW_DEV_AUTH=false`
- [ ] Clerk publishable + secret keys set on web
- [ ] `CLERK_JWKS_URL` + `CLERK_ISSUER` set on API
- [ ] Dashboard redirects unauthenticated users to `/sign-in`
- [ ] `Authorization: Bearer dev` returns **401**
- [ ] Missing `Authorization` on `/api/v1/projects` returns **401**
- [ ] `ENCRYPTION_KEY` is unique (not the repo example)
- [ ] `CORS_ORIGINS` is only your staging frontend origin(s)
- [ ] Stripe test checkout completes and webhook updates `billing_customers`
- [ ] `/api/v1/health` and `/api/v1/ready` succeed
- [ ] `/api/v1/metrics` scrapeable by Prometheus (optional)

## 7) After deploy — first agent smoke

1. Sign in via Clerk on staging web
2. Open `/dashboard/assistant`
3. Run a short `chat` prompt
4. Confirm conversation appears in history (Postgres persistence)
