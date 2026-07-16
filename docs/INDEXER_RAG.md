# Indexer & RAG

## Qdrant RAG

The API boots a Qdrant-backed RAG store when `QDRANT_URL` is reachable; otherwise it falls back to in-memory seed docs.

```bash
docker compose -f infrastructure/docker/docker-compose.yml up -d qdrant
cargo run -p tempoforge-api
```

Collections seeded: `tempo`, `security`.

Embeddings are deterministic local hashed vectors (swap later for hosted embedding APIs without changing agents).

## Indexer worker

```bash
pnpm indexer
# or
cargo run -p tempoforge-api --bin indexer
```

Persists:
- `indexer_state` cursor per network
- `indexed_blocks`
- `transactions` rows from Tempo RPC

Configure with `TEMPO_DEFAULT_NETWORK`, `INDEXER_POLL_MS`, and optional `INDEXER_START_BLOCK`.
