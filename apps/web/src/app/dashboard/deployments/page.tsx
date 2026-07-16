import { Topbar } from "@/components/dashboard/topbar";

export default function DeploymentsPage() {
  return (
    <>
      <Topbar title="Deployments" />
      <div className="p-6">
        <div className="glass rounded-3xl p-10 text-center">
          <h2 className="display text-xl font-semibold">Track Tempo deploys</h2>
          <p className="mt-2 text-sm text-white/60">
            Plan Foundry scripts for mainnet, Moderato testnet, or local anvil — then record tx hashes.
          </p>
        </div>
      </div>
    </>
  );
}
