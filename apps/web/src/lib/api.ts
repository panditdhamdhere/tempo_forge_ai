import type {
  AgentResponse,
  AnalyticsDashboard,
  ApiResponse,
  AuditReport,
  Project,
} from "@tempoforge/types";
import { getApiBase } from "./utils";

async function request<T>(
  path: string,
  init: RequestInit = {},
  token = "dev",
): Promise<T> {
  const res = await fetch(`${getApiBase()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
      ...(init.headers ?? {}),
    },
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
  listProjects: () => request<Project[]>("/api/v1/projects"),
  createProject: (body: { name: string; slug: string; description?: string }) =>
    request<Project>("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify(body),
    }),
  runAgent: (agent: string, prompt: string) =>
    request<AgentResponse>(`/api/v1/ai/agents/${agent}`, {
      method: "POST",
      body: JSON.stringify({ prompt }),
    }),
  audit: (title: string, source: string) =>
    request<{ report: AuditReport }>("/api/v1/audit", {
      method: "POST",
      body: JSON.stringify({ title, source, use_ai: true }),
    }),
  analytics: () => request<AnalyticsDashboard>("/api/v1/analytics/dashboard"),
  explorerTx: (hash: string) =>
    request<unknown>(`/api/v1/explorer/tx/${hash}`, {}, ""),
};
