# TempoForge contracts

Foundry workspace with OpenZeppelin v5 and example `TempoForgeToken`.

## Setup

```bash
cd contracts
forge install foundry-rs/forge-std@v1.9.4 OpenZeppelin/openzeppelin-contracts@v5.2.0 --no-git --shallow
forge build
forge test
```

`pnpm setup` / `scripts/setup.sh` also installs these deps when Foundry is present.

## Deploy to Tempo Moderato testnet

```bash
export PRIVATE_KEY=0x...
export TEMPO_TESTNET_RPC=https://rpc.moderato.tempo.xyz
forge script script/DeployTempoForgeToken.s.sol \
  --rpc-url $TEMPO_TESTNET_RPC \
  --broadcast
```
