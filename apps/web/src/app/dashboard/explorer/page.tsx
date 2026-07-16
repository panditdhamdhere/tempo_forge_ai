"use client";

import { FormEvent, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";

export default function ExplorerPage() {
  const [hash, setHash] = useState("");
  const lookup = useMutation({
    mutationFn: () => api.explorerTx(hash.trim()),
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    lookup.mutate();
  };

  return (
    <>
      <Topbar title="Tempo Explorer" />
      <div className="space-y-6 p-6">
        <form onSubmit={onSubmit} className="glass flex flex-col gap-3 rounded-3xl p-5 md:flex-row">
          <input
            value={hash}
            onChange={(e) => setHash(e.target.value)}
            placeholder="0x… transaction hash"
            className="h-11 flex-1 rounded-xl border border-white/10 bg-black/20 px-4 font-mono text-sm outline-none focus:border-[var(--accent)]"
          />
          <Button type="submit" disabled={lookup.isPending || hash.length < 66}>
            {lookup.isPending ? "Fetching…" : "Inspect"}
          </Button>
        </form>
        {lookup.data != null && (
          <pre className="glass overflow-auto rounded-3xl p-5 text-xs text-white/80">
            {JSON.stringify(lookup.data, null, 2)}
          </pre>
        )}
        {lookup.isError && (
          <p className="text-sm text-[var(--ember)]">{(lookup.error as Error).message}</p>
        )}
      </div>
    </>
  );
}
