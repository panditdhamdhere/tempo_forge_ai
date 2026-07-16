import Link from "next/link";
import { PRICING_TIERS } from "@tempoforge/types";
import { SiteHeader } from "@/components/landing/site-header";
import { Button } from "@/components/ui/button";

export default function PricingPage() {
  return (
    <main>
      <SiteHeader />
      <section className="mx-auto max-w-6xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold md:text-5xl">Pricing</h1>
        <p className="mt-4 max-w-2xl text-white/65">
          Start free on Moderato testnet. Upgrade when your team ships production Tempo apps.
        </p>
        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {PRICING_TIERS.map((tier) => (
            <article key={tier.id} className="glass rounded-3xl p-8">
              <h2 className="display text-2xl font-semibold">{tier.name}</h2>
              <p className="mt-4 text-4xl font-semibold">${tier.price}</p>
              <p className="mt-3 text-white/65">{tier.description}</p>
              <ul className="mt-6 space-y-2 text-sm text-white/75">
                {tier.features.map((f) => (
                  <li key={f}>• {f}</li>
                ))}
              </ul>
              <Button asChild className="mt-8 w-full">
                <Link href="/dashboard">Get started</Link>
              </Button>
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}
