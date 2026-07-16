"use client";

import { FormEvent, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

export default function DeploymentsPage() {
  const api = useAuthedApi();
  const qc = useQueryClient();
  const projects = useQuery({ queryKey: ["projects"], queryFn: () => api.listProjects() });
  const deployments = useQuery({
    queryKey: ["deployments"],
    queryFn: () => api.listDeployments(),
  });

  const [projectId, setProjectId] = useState("");
  const [contractName, setContractName] = useState("TempoForgeToken");
  const [network, setNetwork] = useState("testnet");

  const create = useMutation({
    mutationFn: () =>
      api.createDeployment({
        project_id: projectId,
        contract_name: contractName,
        network,
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["deployments"] }),
  });

  const update = useMutation({
    mutationFn: (input: {
      id: string;
      status: string;
      tx_hash?: string;
      address?: string;
    }) =>
      api.updateDeployment(input.id, {
        status: input.status,
        tx_hash: input.tx_hash,
        address: input.address,
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["deployments"] }),
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!projectId) return;
    create.mutate();
  };

  return (
    <>
      <Topbar title="Deployments" />
      <div className="space-y-6 p-6">
        <form onSubmit={onSubmit} className="glass grid gap-3 rounded-3xl p-5 md:grid-cols-4">
          <select
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
            className="h-11 rounded-xl border border-white/10 bg-black/20 px-3"
            required
          >
            <option value="">Select project</option>
            {projects.data?.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <input
            value={contractName}
            onChange={(e) => setContractName(e.target.value)}
            placeholder="Contract name"
            className="h-11 rounded-xl border border-white/10 bg-black/20 px-4"
          />
          <select
            value={network}
            onChange={(e) => setNetwork(e.target.value)}
            className="h-11 rounded-xl border border-white/10 bg-black/20 px-3"
          >
            <option value="testnet">testnet</option>
            <option value="mainnet">mainnet</option>
            <option value="local">local</option>
          </select>
          <Button type="submit" disabled={create.isPending || !projectId}>
            {create.isPending ? "Saving…" : "Track deployment"}
          </Button>
        </form>

        {!deployments.data?.length && !deployments.isLoading && (
          <div className="glass rounded-3xl p-10 text-center">
            <h2 className="display text-xl font-semibold">No deployments yet</h2>
            <p className="mt-2 text-sm text-white/60">
              Create a project, then track Foundry deploys for Tempo networks.
            </p>
          </div>
        )}

        <div className="space-y-3">
          {deployments.data?.map((d) => (
            <article key={d.id} className="glass rounded-2xl p-5">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 className="font-semibold">
                    {String(d.artifact.contract_name ?? "Contract")} · {d.network}
                  </h3>
                  <p className="mt-1 text-xs text-white/45">{d.id}</p>
                </div>
                <span className="rounded-full bg-white/10 px-3 py-1 text-xs uppercase">
                  {d.status}
                </span>
              </div>
              <p className="mt-3 font-mono text-xs text-white/60">
                tx: {d.tx_hash ?? "—"} · address: {d.address ?? "—"}
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={update.isPending}
                  onClick={() =>
                    update.mutate({
                      id: d.id,
                      status: "submitted",
                      tx_hash: d.tx_hash ?? `0x${d.id.replace(/-/g, "").slice(0, 64)}`,
                    })
                  }
                >
                  Mark submitted
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  disabled={update.isPending}
                  onClick={() =>
                    update.mutate({
                      id: d.id,
                      status: "confirmed",
                      address: d.address ?? "0x00000000000000000000000000000000000000a1",
                    })
                  }
                >
                  Mark confirmed
                </Button>
              </div>
            </article>
          ))}
        </div>
      </div>
    </>
  );
}
