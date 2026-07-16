export const TEMPO_NETWORKS = {
  mainnet: {
    name: "Tempo Mainnet",
    rpcUrl: "https://rpc.tempo.xyz",
    chainId: 4242,
  },
  testnet: {
    name: "Tempo Moderato",
    rpcUrl: "https://rpc.moderato.tempo.xyz",
    chainId: 42431,
  },
} as const;

export const API_DEFAULT_URL = "http://localhost:8080";
