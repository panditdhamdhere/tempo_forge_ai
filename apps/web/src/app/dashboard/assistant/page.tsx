"use client";

import { FormEvent, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

const agents = [
  "planner",
  "codegen",
  "auditor",
  "debugger",
  "docs",
  "tests",
  "chat",
] as const;

export default function AssistantPage() {
  const api = useAuthedApi();
  const [agent, setAgent] = useState<(typeof agents)[number]>("codegen");
  const [prompt, setPrompt] = useState(
    "Create an ERC20 with staking for Tempo testnet using OpenZeppelin.",
  );
  const run = useMutation({
    mutationFn: () => api.runAgent(agent, prompt),
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    run.mutate();
  };

  return (
    <>
      <Topbar title="AI Assistant" />
      <div className="grid gap-6 p-6 lg:grid-cols-[280px_1fr]">
        <aside className="glass h-fit rounded-3xl p-4">
          <p className="text-xs uppercase tracking-[0.18em] text-white/45">Agent</p>
          <div className="mt-3 space-y-1">
            {agents.map((a) => (
              <button
                key={a}
                type="button"
                onClick={() => setAgent(a)}
                className={`block w-full rounded-xl px-3 py-2 text-left text-sm ${
                  agent === a ? "bg-white/10 text-white" : "text-white/60 hover:bg-white/5"
                }`}
              >
                {a}
              </button>
            ))}
          </div>
        </aside>
        <section className="space-y-4">
          <form onSubmit={onSubmit} className="glass rounded-3xl p-5">
            <textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              rows={6}
              className="w-full resize-y rounded-2xl border border-white/10 bg-black/20 p-4 outline-none focus:border-[var(--accent)]"
            />
            <div className="mt-4 flex justify-end">
              <Button type="submit" disabled={run.isPending}>
                {run.isPending ? "Thinking…" : "Run agent"}
              </Button>
            </div>
          </form>
          {run.data && (
            <article className="glass rounded-3xl p-5">
              <p className="text-xs text-white/45">
                model {run.data.model} · {run.data.usage.total_tokens} tokens
              </p>
              <pre className="mt-4 whitespace-pre-wrap text-sm leading-relaxed text-white/85">
                {run.data.content}
              </pre>
              {run.data.files.length > 0 && (
                <div className="mt-6 space-y-3">
                  <h3 className="font-semibold">Generated files</h3>
                  {run.data.files.map((f) => (
                    <details key={f.path} className="rounded-xl border border-white/10 p-3">
                      <summary className="cursor-pointer text-sm text-[var(--accent)]">
                        {f.path}
                      </summary>
                      <pre className="mt-3 overflow-auto text-xs text-white/75">
                        {f.content}
                      </pre>
                    </details>
                  ))}
                </div>
              )}
            </article>
          )}
          {run.isError && (
            <p className="text-sm text-[var(--ember)]">
              {(run.error as Error).message}
            </p>
          )}
        </section>
      </div>
    </>
  );
}
