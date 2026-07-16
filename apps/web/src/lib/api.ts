import type {
  AgentResponse,
  AnalyticsDashboard,
  ApiResponse,
  AuditReport,
  Project,
} from "@tempoforge/types";
import { getApiBase } from "./utils";

export type BillingStatus = {
  plan: string;
  status: string;
  stripe_customer_id?: string | null;
  stripe_subscription_id?: string | null;
  current_period_end?: string | null;
};

async function request<T>(
  path: string,
  init: RequestInit = {},
  token = "dev",
): Promise<T> {
  const headers: Record<string, string> = {
    ...(init.headers as Record<string, string> | undefined),
  };

  if (!(init.body instanceof FormData)) {
    headers["Content-Type"] = headers["Content-Type"] ?? "application/json";
  }

  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const res = await fetch(`${getApiBase()}${path}`, {
    ...init,
    headers,
    cache: "no-store",
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(body || `Request failed: ${res.status}`);
  }

  const json = (await res.json()) as ApiResponse<T> | T;
  if (json && typeof json === "object" && "data" in json) {
    return (json as ApiResponse<T>).data;
  }
  return json as T;
}

export const api = {
  health: () => request<{ status: string }>("/api/v1/health", {}, ""),
  listProjects: (token?: string) =>
    request<Project[]>("/api/v1/projects", {}, token),
  createProject: (
    body: { name: string; slug: string; description?: string },
    token?: string,
  ) =>
    request<Project>(
      "/api/v1/projects",
      { method: "POST", body: JSON.stringify(body) },
      token,
    ),
  runAgent: (agent: string, prompt: string, token?: string) =>
    request<AgentResponse>(
      `/api/v1/ai/agents/${agent}`,
      { method: "POST", body: JSON.stringify({ prompt }) },
      token,
    ),
  audit: (title: string, source: string, token?: string) =>
    request<{ report: AuditReport }>(
      "/api/v1/audit",
      {
        method: "POST",
        body: JSON.stringify({ title, source, use_ai: true }),
      },
      token,
    ),
  analytics: (token?: string) =>
    request<AnalyticsDashboard>("/api/v1/analytics/dashboard", {}, token),
  explorerTx: (hash: string) =>
    request<unknown>(`/api/v1/explorer/tx/${hash}`, {}, ""),
  billingStatus: (token?: string) =>
    request<BillingStatus>("/api/v1/billing/status", {}, token),
  createCheckout: (priceId: string, token?: string) =>
    request<{ checkout_url: string; session_id: string }>(
      "/api/v1/billing/checkout",
      { method: "POST", body: JSON.stringify({ price_id: priceId }) },
      token,
    ),
  createPortal: (token?: string) =>
    request<{ portal_url: string }>(
      "/api/v1/billing/portal",
      { method: "POST", body: JSON.stringify({}) },
      token,
    ),
};
