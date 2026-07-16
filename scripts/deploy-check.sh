#!/usr/bin/env bash
# Validates staging/production env before deploy.
set -euo pipefail

ENV_FILE="${1:-.env.staging}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing $ENV_FILE"
  echo "Copy .env.staging.example → $ENV_FILE and fill secrets."
  exit 1
fi

# shellcheck disable=SC1090
set -a
source "$ENV_FILE"
set +a

fail=0
need() {
  local key="$1"
  if [[ -z "${!key:-}" ]]; then
    echo "FAIL  $key is empty"
    fail=1
  else
    echo "OK    $key"
  fi
}

echo "==> Checking required staging secrets in $ENV_FILE"
need APP_ENV
need APP_URL
need NEXT_PUBLIC_API_URL
need CORS_ORIGINS
need DATABASE_URL
need ENCRYPTION_KEY
need CLERK_JWKS_URL
need CLERK_ISSUER
need NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY
need CLERK_SECRET_KEY
need GROQ_API_KEY

if [[ "${APP_ENV:-}" != "staging" && "${APP_ENV:-}" != "production" ]]; then
  echo "FAIL  APP_ENV must be staging or production (got: ${APP_ENV:-})"
  fail=1
fi

if [[ "${ALLOW_DEV_AUTH:-}" == "true" || "${ALLOW_DEV_AUTH:-}" == "1" ]]; then
  echo "FAIL  ALLOW_DEV_AUTH must be false in staging/production"
  fail=1
fi

if [[ "${ENCRYPTION_KEY:-}" == "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" ]]; then
  echo "FAIL  ENCRYPTION_KEY is still the example default"
  fail=1
fi

if [[ "${APP_URL:-}" == *"localhost"* ]]; then
  echo "FAIL  APP_URL must be a public HTTPS origin (not localhost)"
  fail=1
fi

if [[ "${CORS_ORIGINS:-}" == *"*"* ]]; then
  echo "FAIL  CORS_ORIGINS must not be wildcard"
  fail=1
fi

echo ""
if [[ "$fail" -ne 0 ]]; then
  echo "Deploy check FAILED — fix the items above."
  exit 1
fi

echo "Deploy check PASSED."
echo "Next:"
echo "  docker compose -f infrastructure/docker/docker-compose.staging.yml --env-file $ENV_FILE up -d --build"
echo "  # Stripe webhook → \${NEXT_PUBLIC_API_URL}/api/v1/billing/webhook"
