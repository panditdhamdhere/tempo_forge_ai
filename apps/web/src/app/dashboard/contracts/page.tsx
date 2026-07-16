import { Topbar } from "@/components/dashboard/topbar";

export default function ContractsPage() {
  return (
    <>
      <Topbar title="Contracts" />
      <div className="p-6">
        <div className="glass rounded-3xl p-10 text-center">
          <h2 className="display text-xl font-semibold">No contracts indexed yet</h2>
          <p className="mt-2 text-sm text-white/60">
            Generate with the AI Assistant or upload Solidity to attach contracts to a project.
          </p>
        </div>
      </div>
    </>
  );
}
