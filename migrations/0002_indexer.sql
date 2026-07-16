CREATE TABLE IF NOT EXISTS indexer_state (
    network TEXT PRIMARY KEY,
    cursor_block BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS indexed_blocks (
    network TEXT NOT NULL,
    number BIGINT NOT NULL,
    hash TEXT,
    tx_count INT NOT NULL DEFAULT 0,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (network, number)
);

CREATE INDEX IF NOT EXISTS idx_indexed_blocks_network_number
    ON indexed_blocks (network, number DESC);
