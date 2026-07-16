# Getting started

## Prerequisites

- Node.js 20+
- pnpm 9+
- Rust 1.82+
- Docker

## One command

```bash
pnpm setup
```

This installs dependencies, starts Postgres/Redis/Qdrant/MinIO, migrates, and seeds.

## Run

```bash
# Terminal 1 — API
cargo run -p tempoforge-api

# Terminal 2 — Web
pnpm --filter @tempoforge/web dev

# Optional CLI
cargo run -p tempoforge-cli -- health
```

Open http://localhost:3000

## Auth

- Local development accepts `Authorization: Bearer dev` (and anonymous in some explorer routes).
- Production: set Clerk `CLERK_JWKS_URL`, `CLERK_ISSUER`, and Next.js publishable/secret keys.

## AI

Set `GROQ_API_KEY` (default provider) or switch `AI_PROVIDER` to `openai` / `local`.

## Tempo RPC

Defaults to Moderato testnet: `https://rpc.moderato.tempo.xyz`.
