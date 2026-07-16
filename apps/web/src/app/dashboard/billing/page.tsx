import { PRICING_TIERS } from "@tempoforge/types";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";

export default function BillingPage() {
  return (
    <>
      <Topbar title="Billing" />
      <div className="grid gap-4 p-6 md:grid-cols-3">
        {PRICING_TIERS.map((tier) => (
          <article key={tier.id} className="glass rounded-3xl p-6">
            <p className="text-sm uppercase tracking-[0.18em] text-white/45">{tier.name}</p>
            <p className="display mt-3 text-3xl font-semibold">${tier.price}</p>
            <p className="mt-2 text-sm text-white/60">{tier.description}</p>
            <Button className="mt-6 w-full" variant={tier.id === "pro" ? "default" : "secondary"}>
              {tier.id === "free" ? "Current plan" : "Upgrade with Stripe"}
            </Button>
          </article>
        ))}
      </div>
    </>
  );
}
