import Link from "next/link";
import { FEATURE_BLURBS } from "@tempoforge/prompts";
import { PRICING_TIERS } from "@tempoforge/types";
import { SiteHeader } from "@/components/landing/site-header";
import { Hero } from "@/components/landing/hero";
import { Button } from "@/components/ui/button";

const testimonials = [
  {
    quote:
      "TempoForge cut our TIP-20 vault launch from two weeks to an afternoon — audits included.",
    name: "Maya Chen",
    role: "Founding Engineer, LanePay",
  },
  {
    quote:
      "The debugger finally speaks Tempo’s fee model. Failed txs stop being black boxes.",
    name: "Omar Farid",
    role: "Protocol Lead, Northmint",
  },
];

const roadmap = [
  { when: "Now", item: "AI codegen, auditor, explorer, analytics dashboard" },
  { when: "Q3", item: "Full indexer + NFT explorer + org SSO" },
  { when: "Q4", item: "Managed verification, multi-region API, enterprise controls" },
];

export default function HomePage() {
  return (
    <main>
      <SiteHeader />
      <Hero />

      <section id="features" className="mx-auto max-w-6xl px-6 py-24">
        <h2 className="display text-3xl font-semibold md:text-4xl">
          One forge. Every Tempo workflow.
        </h2>
        <p className="mt-3 max-w-2xl text-white/65">
          From first prompt to mainnet deploy — built for TIP-20 payments, Tempo
          RPC quirks, and production security.
        </p>
        <div className="mt-12 grid gap-10 md:grid-cols-3">
          {FEATURE_BLURBS.map((feature) => (
            <article key={feature.title}>
              <h3 className="display text-xl font-semibold text-[var(--accent)]">
                {feature.title}
              </h3>
              <p className="mt-3 text-white/70">{feature.body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="border-y border-white/10 bg-black/20 py-24">
        <div className="mx-auto max-w-6xl px-6">
          <h2 className="display text-3xl font-semibold">Pricing that scales with shipping.</h2>
          <div className="mt-12 grid gap-8 md:grid-cols-3">
            {PRICING_TIERS.map((tier) => (
              <div key={tier.id} className="glass rounded-3xl p-8">
                <p className="text-sm uppercase tracking-[0.2em] text-white/50">
                  {tier.name}
                </p>
                <p className="display mt-4 text-4xl font-semibold">
                  ${tier.price}
                  <span className="text-base font-normal text-white/50">/mo</span>
                </p>
                <p className="mt-3 text-white/65">{tier.description}</p>
                <ul className="mt-6 space-y-2 text-sm text-white/75">
                  {tier.features.map((f) => (
                    <li key={f}>• {f}</li>
                  ))}
                </ul>
                <Button asChild className="mt-8 w-full">
                  <Link href="/dashboard">Choose {tier.name}</Link>
                </Button>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-6 py-24">
        <h2 className="display text-3xl font-semibold">Builders already forging.</h2>
        <div className="mt-10 grid gap-8 md:grid-cols-2">
          {testimonials.map((t) => (
            <blockquote key={t.name} className="glass rounded-3xl p-8">
              <p className="text-lg text-white/85">“{t.quote}”</p>
              <footer className="mt-6 text-sm text-white/55">
                {t.name} — {t.role}
              </footer>
            </blockquote>
          ))}
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-6 pb-24">
        <h2 className="display text-3xl font-semibold">Roadmap</h2>
        <div className="mt-8 space-y-4">
          {roadmap.map((item) => (
            <div
              key={item.item}
              className="flex flex-col gap-2 border-l-2 border-[var(--accent)] pl-5 md:flex-row md:items-baseline md:gap-8"
            >
              <span className="text-sm font-semibold text-[var(--accent)]">
                {item.when}
              </span>
              <span className="text-white/75">{item.item}</span>
            </div>
          ))}
        </div>
      </section>

      <footer className="border-t border-white/10 py-10">
        <div className="mx-auto flex max-w-6xl flex-col gap-4 px-6 text-sm text-white/50 md:flex-row md:items-center md:justify-between">
          <p>© {new Date().getFullYear()} TempoForge AI</p>
          <div className="flex gap-6">
            <Link href="/docs">Docs</Link>
            <Link href="/blog">Blog</Link>
            <Link href="/pricing">Pricing</Link>
            <Link href="/dashboard">Dashboard</Link>
          </div>
        </div>
      </footer>
    </main>
  );
}
