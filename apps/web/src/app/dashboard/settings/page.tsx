import { Topbar } from "@/components/dashboard/topbar";

export default function SettingsPage() {
  return (
    <>
      <Topbar title="Settings" />
      <div className="space-y-4 p-6">
        <section className="glass rounded-3xl p-6">
          <h2 className="font-semibold">API keys</h2>
          <p className="mt-2 text-sm text-white/60">
            Create hashed org API keys for CI and SDK access. Keys are shown once at creation.
          </p>
        </section>
        <section className="glass rounded-3xl p-6">
          <h2 className="font-semibold">Tempo RPC</h2>
          <p className="mt-2 text-sm text-white/60">
            Default: Moderato testnet <code className="text-[var(--accent)]">https://rpc.moderato.tempo.xyz</code>
          </p>
        </section>
      </div>
    </>
  );
}
