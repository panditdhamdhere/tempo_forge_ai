#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> TempoForge AI setup"

if [ ! -f .env ]; then
  cp .env.example .env
  echo "Created .env from .env.example"
fi

echo "==> Installing JS dependencies"
pnpm install

echo "==> Starting infrastructure (Postgres, Redis, Qdrant, MinIO)"
docker compose -f infrastructure/docker/docker-compose.yml up -d

echo "==> Waiting for Postgres"
for i in {1..30}; do
  if docker compose -f infrastructure/docker/docker-compose.yml exec -T postgres pg_isready -U tempoforge -d tempoforge >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "==> Running migrations + seed"
export DATABASE_URL="${DATABASE_URL:-postgres://tempoforge:tempoforge@localhost:5432/tempoforge}"
cargo run -p tempoforge-api --bin seed

echo "==> Checking Rust workspace"
cargo check --workspace

echo ""
echo "Setup complete."
echo "  API:  cargo run -p tempoforge-api"
echo "  Web:  pnpm --filter @tempoforge/web dev"
echo "  CLI:  cargo run -p tempoforge-cli -- health"
echo "  Open: http://localhost:3000"
