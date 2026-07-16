import { SiteHeader } from "@/components/landing/site-header";

export default function ApiDocPage() {
  return (
    <main>
      <SiteHeader />
      <article className="mx-auto max-w-3xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold">REST API</h1>
        <p className="mt-4 text-white/65">
          OpenAPI document: <code className="text-[var(--accent)]">GET /api/v1/openapi.json</code>
        </p>
        <ul className="mt-8 space-y-3 text-white/75">
          <li>POST /api/v1/ai/agents/{"{agent}"} — run planner, codegen, auditor, …</li>
          <li>POST /api/v1/audit — static + AI security report</li>
          <li>GET /api/v1/explorer/tx/{"{hash}"} — Tempo transaction view</li>
          <li>GET /api/v1/analytics/dashboard — chain analytics snapshot</li>
        </ul>
      </article>
    </main>
  );
}
