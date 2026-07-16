# TempoForge AI

**The AI-powered developer platform for Tempo Blockchain.**

Cursor + Alchemy + Tenderly + Vercel — purpose-built for Tempo developers.

## Product

- AI Smart Contract Generator (Solidity, Foundry tests, scripts, docs)
- AI Auditor (static detectors + LLM findings with diffs)
- AI Debugger & Chat
- Tempo Explorer & Analytics
- Deployment planning & SDK generation
- Dashboard with command palette, projects, billing

## Monorepo

```
apps/web          Next.js App Router
apps/api          Axum REST API
apps/cli          Developer CLI
crates/*          Domain services (AI, blockchain, security, explorer, …)
packages/*        Shared TS types, prompts, config
migrations/       PostgreSQL schema
infrastructure/   Docker Compose (local)
contracts/        Foundry example contracts
```

## Quick start

```bash
cp .env.example .env
pnpm setup
cargo run -p tempoforge-api
pnpm --filter @tempoforge/web dev
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md), and [docs/STAGING_DEPLOY.md](docs/STAGING_DEPLOY.md).

## API

Base URL: `http://localhost:8080/api/v1`

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness |
| GET | `/openapi.json` | OpenAPI document |
| GET/POST | `/projects` | Projects |
| POST | `/ai/agents/{agent}` | Run AI agent |
| POST | `/audit` | Security audit |
| GET | `/explorer/tx/{hash}` | Transaction explorer |
| GET | `/analytics/dashboard` | Analytics |

## Tempo

- Mainnet: `https://rpc.tempo.xyz`
- Testnet: `https://rpc.moderato.tempo.xyz` (chain id `42431`)
- TIP-20 for balances/fees — do not trust `eth_getBalance`

## License

MIT
