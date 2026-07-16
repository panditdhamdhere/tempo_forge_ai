import { SiteHeader } from "@/components/landing/site-header";

export default function GettingStartedDoc() {
  return (
    <main>
      <SiteHeader />
      <article className="mx-auto max-w-3xl px-6 pb-24 pt-32 prose-invert">
        <h1 className="display text-4xl font-semibold">Getting started</h1>
        <ol className="mt-8 list-decimal space-y-4 pl-5 text-white/75">
          <li>
            Copy env: <code className="text-[var(--accent)]">cp .env.example .env</code>
          </li>
          <li>
            Run setup: <code className="text-[var(--accent)]">pnpm setup</code>
          </li>
          <li>
            Start API: <code className="text-[var(--accent)]">cargo run -p tempoforge-api</code>
          </li>
          <li>
            Start web: <code className="text-[var(--accent)]">pnpm --filter @tempoforge/web dev</code>
          </li>
        </ol>
      </article>
    </main>
  );
}
