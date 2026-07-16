"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

export default function DashboardPage() {
  const api = useAuthedApi();
  const projects = useQuery({ queryKey: ["projects"], queryFn: () => api.listProjects() });
  const analytics = useQuery({ queryKey: ["analytics"], queryFn: () => api.analytics() });

  return (
    <>
      <Topbar title="Overview" />
      <div className="space-y-8 p-6">
        <section className="grid gap-4 md:grid-cols-3">
          <Stat
            label="Latest Tempo block"
            value={
              analytics.isLoading
                ? "…"
                : analytics.data?.network.latest_block.toLocaleString() ?? "—"
            }
          />
          <Stat
            label="Projects"
            value={projects.isLoading ? "…" : String(projects.data?.length ?? 0)}
          />
          <Stat label="Network" value={analytics.data?.network.network ?? "testnet"} />
        </section>

        <section className="glass rounded-3xl p-6">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h2 className="display text-xl font-semibold">Jump back in</h2>
              <p className="mt-1 text-sm text-white/60">
                Generate, audit, or explore Tempo without leaving the forge.
              </p>
            </div>
            <Button asChild>
              <Link href="/dashboard/assistant">Open AI Assistant</Link>
            </Button>
          </div>
          <div className="mt-6 grid gap-3 md:grid-cols-3">
            {[
              ["Generate a TIP-20 staking vault", "/dashboard/assistant"],
              ["Audit a contract", "/dashboard/auditor"],
              ["Inspect a transaction", "/dashboard/explorer"],
            ].map(([label, href]) => (
              <Link
                key={label}
                href={href}
                className="rounded-2xl border border-white/10 bg-white/5 px-4 py-4 text-sm text-white/80 transition hover:bg-white/10"
              >
                {label}
              </Link>
            ))}
          </div>
        </section>

        <section>
          <h2 className="display text-xl font-semibold">Projects</h2>
          {projects.isLoading && (
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              <div className="h-28 animate-pulse rounded-2xl bg-white/5" />
              <div className="h-28 animate-pulse rounded-2xl bg-white/5" />
            </div>
          )}
          {projects.isError && (
            <p className="mt-4 text-sm text-[var(--ember)]">
              API unreachable. Start Docker + `cargo run -p tempoforge-api`.
            </p>
          )}
          {projects.data?.length === 0 && (
            <div className="glass mt-4 rounded-3xl p-10 text-center">
              <p className="display text-lg">No projects yet</p>
              <p className="mt-2 text-sm text-white/60">
                Create your first Tempo project to store contracts and audits.
              </p>
              <Button asChild className="mt-6">
                <Link href="/dashboard/projects">Create project</Link>
              </Button>
            </div>
          )}
          <div className="mt-4 grid gap-3 md:grid-cols-2">
            {projects.data?.map((p) => (
              <article key={p.id} className="glass rounded-2xl p-5">
                <h3 className="font-semibold">{p.name}</h3>
                <p className="mt-1 text-sm text-white/55">{p.description || p.slug}</p>
              </article>
            ))}
          </div>
        </section>
      </div>
    </>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="glass rounded-2xl p-5">
      <p className="text-xs uppercase tracking-[0.18em] text-white/45">{label}</p>
      <p className="display mt-3 text-3xl font-semibold">{value}</p>
    </div>
  );
}
