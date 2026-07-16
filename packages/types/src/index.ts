export type PlanTier = "free" | "pro" | "team" | "enterprise";
export type NetworkEnv = "mainnet" | "testnet" | "local";
export type Severity = "critical" | "high" | "medium" | "low" | "info";

export type AgentKind =
  | "planner"
  | "code_generator"
  | "auditor"
  | "debugger"
  | "architect"
  | "documentation_writer"
  | "test_generator"
  | "deployment_assistant"
  | "chat";

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}

export interface ApiResponse<T> {
  data: T;
}

export interface Project {
  id: string;
  org_id: string;
  name: string;
  slug: string;
  description: string;
  created_at: string;
}

export interface Finding {
  severity: Severity;
  title: string;
  description: string;
  location?: string | null;
  recommendation: string;
  diff?: string | null;
}

export interface AuditReport {
  id: string;
  title: string;
  source_hash: string;
  findings: Finding[];
  summary: {
    critical: number;
    high: number;
    medium: number;
    low: number;
    info: number;
  };
  ai_narrative?: string | null;
  created_at: string;
}

export interface AgentResponse {
  request_id: string;
  agent: AgentKind;
  content: string;
  files: Array<{ path: string; content: string; language: string }>;
  findings: Finding[];
  follow_ups: string[];
  model: string;
  created_at: string;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface AnalyticsDashboard {
  network: {
    network: string;
    chain_id: number;
    latest_block: number;
    rpc_url: string;
    sampled_at: string;
    notes: string[];
  };
  gas_proxy_series: Array<{ timestamp: string; value: number; label: string }>;
  tps_proxy_series: Array<{ timestamp: string; value: number; label: string }>;
  wallet_growth_series: Array<{ timestamp: string; value: number; label: string }>;
}

export const PRICING_TIERS = [
  {
    id: "free" as const,
    name: "Free",
    price: 0,
    description: "Explore TempoForge with core AI tools.",
    features: ["3 projects", "AI chat", "Static auditor", "Testnet explorer"],
  },
  {
    id: "pro" as const,
    name: "Pro",
    price: 49,
    description: "For indie Tempo builders shipping production apps.",
    features: [
      "Unlimited projects",
      "Codegen + tests",
      "AI auditor + debugger",
      "Deployment tracking",
      "Priority models",
    ],
  },
  {
    id: "team" as const,
    name: "Team",
    price: 199,
    description: "Collaboration, SSO-ready orgs, and shared audits.",
    features: [
      "Everything in Pro",
      "Organization RBAC",
      "Shared templates",
      "API keys",
      "Stripe billing seats",
    ],
  },
] as const;
