"use client";

import { FormEvent, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { getApiBase } from "@/lib/utils";
import { useApiToken } from "@/lib/auth-token";

type ApiKeyView = {
  id: string;
  name: string;
  key_prefix: string;
  revoked_at?: string | null;
  created_at: string;
};

export default function SettingsPage() {
  const { getToken } = useApiToken();
  const qc = useQueryClient();
  const [name, setName] = useState("ci");
  const [createdRaw, setCreatedRaw] = useState<string | null>(null);

  const keys = useQuery({
    queryKey: ["api-keys"],
    queryFn: async () => {
      const token = await getToken();
      const res = await fetch(`${getApiBase()}/api/v1/keys`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!res.ok) throw new Error(await res.text());
      const json = await res.json();
      return json.data as ApiKeyView[];
    },
  });

  const create = useMutation({
    mutationFn: async () => {
      const token = await getToken();
      const res = await fetch(`${getApiBase()}/api/v1/keys`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ name }),
      });
      if (!res.ok) throw new Error(await res.text());
      return res.json();
    },
    onSuccess: (json) => {
      setCreatedRaw(json.data.raw_key);
      qc.invalidateQueries({ queryKey: ["api-keys"] });
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    create.mutate();
  };

  return (
    <>
      <Topbar title="Settings" />
      <div className="space-y-4 p-6">
        <section className="glass rounded-3xl p-6">
          <h2 className="font-semibold">API keys</h2>
          <p className="mt-2 text-sm text-white/60">
            Hashed org keys for CI and SDK access. The raw secret is shown once.
          </p>
          <form onSubmit={onSubmit} className="mt-4 flex flex-col gap-3 md:flex-row">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="h-11 flex-1 rounded-xl border border-white/10 bg-black/20 px-4"
              placeholder="Key name"
            />
            <Button type="submit" disabled={create.isPending}>
              {create.isPending ? "Creating…" : "Create key"}
            </Button>
          </form>
          {createdRaw && (
            <pre className="mt-4 overflow-auto rounded-xl bg-black/40 p-3 text-xs text-[var(--accent)]">
              {createdRaw}
            </pre>
          )}
          <div className="mt-4 space-y-2">
            {keys.data?.map((k) => (
              <div
                key={k.id}
                className="flex items-center justify-between rounded-xl border border-white/10 px-3 py-2 text-sm"
              >
                <span>
                  {k.name} · {k.key_prefix}…
                </span>
                <span className="text-xs text-white/45">
                  {k.revoked_at ? "revoked" : "active"}
                </span>
              </div>
            ))}
          </div>
        </section>
        <section className="glass rounded-3xl p-6">
          <h2 className="font-semibold">Tempo RPC</h2>
          <p className="mt-2 text-sm text-white/60">
            Default: Moderato testnet{" "}
            <code className="text-[var(--accent)]">
              https://rpc.moderato.tempo.xyz
            </code>
          </p>
        </section>
        <section className="glass rounded-3xl p-6">
          <h2 className="font-semibold">Observability</h2>
          <p className="mt-2 text-sm text-white/60">
            Prometheus scrape target:{" "}
            <code className="text-[var(--accent)]">/api/v1/metrics</code>
          </p>
        </section>
      </div>
    </>
  );
}
