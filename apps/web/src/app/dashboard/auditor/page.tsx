"use client";

import { FormEvent, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

const sample = `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract Vault {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    function withdraw() external {
        uint256 amount = balances[msg.sender];
        (bool ok,) = msg.sender.call{value: amount}("");
        require(ok);
        balances[msg.sender] = 0;
    }

    function adminOnly() external view {
        require(tx.origin == msg.sender);
    }
}
`;

export default function AuditorPage() {
  const api = useAuthedApi();
  const [source, setSource] = useState(sample);
  const audit = useMutation({
    mutationFn: () => api.audit("Dashboard audit", source),
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    audit.mutate();
  };

  const report = audit.data?.report;

  return (
    <>
      <Topbar title="AI Auditor" />
      <div className="grid gap-6 p-6 xl:grid-cols-2">
        <form onSubmit={onSubmit} className="glass rounded-3xl p-5">
          <textarea
            value={source}
            onChange={(e) => setSource(e.target.value)}
            rows={22}
            className="w-full rounded-2xl border border-white/10 bg-black/30 p-4 font-mono text-xs outline-none focus:border-[var(--accent)]"
          />
          <Button type="submit" className="mt-4" disabled={audit.isPending}>
            {audit.isPending ? "Auditing…" : "Run audit"}
          </Button>
        </form>
        <section className="space-y-4">
          {!report && !audit.isPending && (
            <div className="glass rounded-3xl p-10 text-center text-white/60">
              Paste Solidity to detect reentrancy, access control, and Tempo-specific risks.
            </div>
          )}
          {report && (
            <>
              <div className="grid grid-cols-5 gap-2">
                {(
                  [
                    ["critical", report.summary.critical],
                    ["high", report.summary.high],
                    ["medium", report.summary.medium],
                    ["low", report.summary.low],
                    ["info", report.summary.info],
                  ] as const
                ).map(([label, value]) => (
                  <div key={label} className="glass rounded-xl p-3 text-center">
                    <p className="text-[10px] uppercase tracking-wider text-white/45">{label}</p>
                    <p className="display mt-1 text-2xl">{value}</p>
                  </div>
                ))}
              </div>
              {report.findings.map((f) => (
                <article key={`${f.title}-${f.location}`} className="glass rounded-2xl p-5">
                  <div className="flex items-center justify-between gap-3">
                    <h3 className="font-semibold">{f.title}</h3>
                    <span className="rounded-full bg-white/10 px-2 py-0.5 text-xs uppercase">
                      {f.severity}
                    </span>
                  </div>
                  <p className="mt-2 text-sm text-white/70">{f.description}</p>
                  {f.location && (
                    <p className="mt-2 font-mono text-xs text-white/45">{f.location}</p>
                  )}
                  <p className="mt-3 text-sm text-[var(--accent)]">{f.recommendation}</p>
                  {f.diff && (
                    <pre className="mt-3 overflow-auto rounded-xl bg-black/40 p-3 text-xs text-white/75">
                      {f.diff}
                    </pre>
                  )}
                </article>
              ))}
            </>
          )}
        </section>
      </div>
    </>
  );
}
