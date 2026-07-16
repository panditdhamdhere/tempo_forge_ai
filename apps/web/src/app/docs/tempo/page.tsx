import { SiteHeader } from "@/components/landing/site-header";

export default function TempoDocPage() {
  return (
    <main>
      <SiteHeader />
      <article className="mx-auto max-w-3xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold">Tempo Blockchain</h1>
        <ul className="mt-8 space-y-4 text-white/75">
          <li>Mainnet RPC: https://rpc.tempo.xyz</li>
          <li>Testnet RPC: https://rpc.moderato.tempo.xyz (chain id 42431)</li>
          <li>eth_getBalance returns a placeholder — use TIP-20 balanceOf</li>
          <li>Fees are paid in TIP-20 stablecoins</li>
          <li>Faucet: cast rpc tempo_fundAddress &lt;addr&gt; --rpc-url https://rpc.moderato.tempo.xyz</li>
        </ul>
      </article>
    </main>
  );
}
