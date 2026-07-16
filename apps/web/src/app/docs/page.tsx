import Link from "next/link";
import { SiteHeader } from "@/components/landing/site-header";

export default function DocsPage() {
  return (
    <main>
      <SiteHeader />
      <section className="mx-auto max-w-3xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold">Documentation</h1>
        <p className="mt-4 text-white/65">
          TempoForge AI is the developer platform for Tempo Blockchain. See the
          repository docs for architecture and local setup.
        </p>
        <div className="mt-10 space-y-6">
          <DocLink href="/docs/getting-started" title="Getting started" body="One-command local setup with Docker, API, and web." />
          <DocLink href="/docs/api" title="REST API" body="Versioned /api/v1 endpoints for agents, audit, explorer, and analytics." />
          <DocLink href="/docs/tempo" title="Tempo notes" body="TIP-20 fees, Moderato faucet, and RPC differences vs Ethereum." />
        </div>
        <p className="mt-10 text-sm text-white/50">
          Full markdown lives in <code>docs/</code> in the monorepo.{" "}
          <Link className="text-[var(--accent)]" href="/dashboard">
            Open the dashboard
          </Link>
          .
        </p>
      </section>
    </main>
  );
}

function DocLink({
  href,
  title,
  body,
}: {
  href: string;
  title: string;
  body: string;
}) {
  return (
    <Link href={href} className="glass block rounded-2xl p-6 transition hover:bg-white/5">
      <h2 className="text-lg font-semibold">{title}</h2>
      <p className="mt-2 text-sm text-white/60">{body}</p>
    </Link>
  );
}
