"use client";

import { useQuery } from "@tanstack/react-query";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Topbar } from "@/components/dashboard/topbar";
import { useAuthedApi } from "@/hooks/use-authed-api";

export default function AnalyticsPage() {
  const api = useAuthedApi();
  const { data, isLoading, isError } = useQuery({
    queryKey: ["analytics"],
    queryFn: () => api.analytics(),
  });

  const series =
    data?.tps_proxy_series.map((p) => ({
      t: new Date(p.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      value: p.value,
    })) ?? [];

  return (
    <>
      <Topbar title="AI Analytics" />
      <div className="space-y-6 p-6">
        {isLoading && <div className="h-72 animate-pulse rounded-3xl bg-white/5" />}
        {isError && (
          <p className="text-sm text-[var(--ember)]">Failed to load analytics.</p>
        )}
        {data && (
          <>
            <div className="grid gap-4 md:grid-cols-3">
              <Metric label="Chain ID" value={String(data.network.chain_id)} />
              <Metric
                label="Latest block"
                value={data.network.latest_block.toLocaleString()}
              />
              <Metric label="RPC" value={data.network.rpc_url.replace("https://", "")} />
            </div>
            <div className="glass rounded-3xl p-5">
              <h2 className="display text-lg font-semibold">TPS proxy</h2>
              <div className="mt-4 h-72">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={series}>
                    <defs>
                      <linearGradient id="tps" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="#3dffb5" stopOpacity={0.45} />
                        <stop offset="100%" stopColor="#3dffb5" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid stroke="rgba(255,255,255,0.06)" />
                    <XAxis dataKey="t" stroke="#6f7f78" fontSize={12} />
                    <YAxis stroke="#6f7f78" fontSize={12} />
                    <Tooltip
                      contentStyle={{
                        background: "#0c1816",
                        border: "1px solid rgba(255,255,255,0.1)",
                        borderRadius: 12,
                      }}
                    />
                    <Area
                      type="monotone"
                      dataKey="value"
                      stroke="#3dffb5"
                      fill="url(#tps)"
                    />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>
          </>
        )}
      </div>
    </>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="glass rounded-2xl p-5">
      <p className="text-xs uppercase tracking-[0.18em] text-white/45">{label}</p>
      <p className="display mt-2 truncate text-xl font-semibold">{value}</p>
    </div>
  );
}
