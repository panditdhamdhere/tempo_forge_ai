"use client";

import { useMutation, useQuery } from "@tanstack/react-query";
import { PRICING_TIERS } from "@tempoforge/types";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

const PRICE_ENV: Record<string, string | undefined> = {
  pro: process.env.NEXT_PUBLIC_STRIPE_PRICE_PRO,
  team: process.env.NEXT_PUBLIC_STRIPE_PRICE_TEAM,
};

export default function BillingPage() {
  const api = useAuthedApi();
  const status = useQuery({
    queryKey: ["billing-status"],
    queryFn: () => api.billingStatus(),
  });

  const checkout = useMutation({
    mutationFn: (priceId: string) => api.createCheckout(priceId),
    onSuccess: (data) => {
      window.location.href = data.checkout_url;
    },
  });

  const portal = useMutation({
    mutationFn: () => api.createPortal(),
    onSuccess: (data) => {
      window.location.href = data.portal_url;
    },
  });

  return (
    <>
      <Topbar title="Billing" />
      <div className="space-y-6 p-6">
        <section className="glass rounded-3xl p-6">
          <p className="text-xs uppercase tracking-[0.18em] text-white/45">
            Current plan
          </p>
          <p className="display mt-2 text-2xl font-semibold capitalize">
            {status.data?.plan ?? (status.isLoading ? "…" : "free")}
          </p>
          <p className="mt-1 text-sm text-white/55">
            Status: {status.data?.status ?? "inactive"}
          </p>
          {status.data?.stripe_customer_id && (
            <Button
              className="mt-4"
              variant="secondary"
              onClick={() => portal.mutate()}
              disabled={portal.isPending}
            >
              {portal.isPending ? "Opening…" : "Manage in Stripe"}
            </Button>
          )}
        </section>

        <div className="grid gap-4 md:grid-cols-3">
          {PRICING_TIERS.map((tier) => {
            const priceId = PRICE_ENV[tier.id];
            const isCurrent = status.data?.plan === tier.id;
            return (
              <article key={tier.id} className="glass rounded-3xl p-6">
                <p className="text-sm uppercase tracking-[0.18em] text-white/45">
                  {tier.name}
                </p>
                <p className="display mt-3 text-3xl font-semibold">
                  ${tier.price}
                </p>
                <p className="mt-2 text-sm text-white/60">{tier.description}</p>
                <Button
                  className="mt-6 w-full"
                  variant={tier.id === "pro" ? "default" : "secondary"}
                  disabled={
                    tier.id === "free" ||
                    isCurrent ||
                    checkout.isPending ||
                    !priceId
                  }
                  onClick={() => priceId && checkout.mutate(priceId)}
                >
                  {tier.id === "free"
                    ? "Included"
                    : isCurrent
                      ? "Current plan"
                      : priceId
                        ? "Upgrade with Stripe"
                        : "Set NEXT_PUBLIC_STRIPE_PRICE_*"}
                </Button>
              </article>
            );
          })}
        </div>
        {checkout.isError && (
          <p className="text-sm text-[var(--ember)]">
            {(checkout.error as Error).message}
          </p>
        )}
      </div>
    </>
  );
}
