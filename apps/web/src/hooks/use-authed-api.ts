"use client";

import { useCallback } from "react";
import { api } from "@/lib/api";
import { useApiToken } from "@/lib/auth-token";

export function useAuthedApi() {
  const { getToken, isLoaded, isSignedIn } = useApiToken();

  const withToken = useCallback(
    async <T,>(fn: (token: string) => Promise<T>) => {
      const token = await getToken();
      return fn(token);
    },
    [getToken],
  );

  return {
    isLoaded,
    isSignedIn,
    listProjects: () => withToken((t) => api.listProjects(t)),
    createProject: (body: { name: string; slug: string; description?: string }) =>
      withToken((t) => api.createProject(body, t)),
    runAgent: (agent: string, prompt: string) =>
      withToken((t) => api.runAgent(agent, prompt, t)),
    audit: (title: string, source: string) =>
      withToken((t) => api.audit(title, source, t)),
    analytics: () => withToken((t) => api.analytics(t)),
    billingStatus: () => withToken((t) => api.billingStatus(t)),
    createCheckout: (priceId: string) =>
      withToken((t) => api.createCheckout(priceId, t)),
    createPortal: () => withToken((t) => api.createPortal(t)),
  };
}
