"use client";

import { FormEvent, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Topbar } from "@/components/dashboard/topbar";
import { Button } from "@/components/ui/button";
import { useAuthedApi } from "@/hooks/use-authed-api";

export default function ProjectsPage() {
  const qc = useQueryClient();
  const api = useAuthedApi();
  const projects = useQuery({ queryKey: ["projects"], queryFn: () => api.listProjects() });
  const [name, setName] = useState("");
  const create = useMutation({
    mutationFn: api.createProject,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["projects"] });
      setName("");
    },
  });

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    const slug = name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/(^-|-$)/g, "");
    if (!slug) return;
    create.mutate({ name, slug, description: "Created from TempoForge dashboard" });
  };

  return (
    <>
      <Topbar title="Projects" />
      <div className="space-y-6 p-6">
        <form onSubmit={onSubmit} className="glass flex flex-col gap-3 rounded-3xl p-5 md:flex-row">
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Project name"
            className="h-11 flex-1 rounded-xl border border-white/10 bg-black/20 px-4 outline-none focus:border-[var(--accent)]"
          />
          <Button type="submit" disabled={create.isPending || !name.trim()}>
            {create.isPending ? "Creating…" : "Create project"}
          </Button>
        </form>
        <div className="grid gap-3 md:grid-cols-2">
          {projects.data?.map((p) => (
            <article key={p.id} className="glass rounded-2xl p-5">
              <h2 className="font-semibold">{p.name}</h2>
              <p className="mt-1 text-sm text-white/55">{p.slug}</p>
            </article>
          ))}
        </div>
      </div>
    </>
  );
}
