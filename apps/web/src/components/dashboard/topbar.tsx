"use client";

import { Bell, PanelLeft, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useUiStore } from "@/store/ui";

export function Topbar({ title }: { title: string }) {
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const setCommandOpen = useUiStore((s) => s.setCommandOpen);

  return (
    <header className="flex items-center justify-between gap-4 border-b border-white/10 px-6 py-4">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" onClick={toggleSidebar} aria-label="Toggle sidebar">
          <PanelLeft className="h-4 w-4" />
        </Button>
        <h1 className="display text-xl font-semibold">{title}</h1>
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          onClick={() => setCommandOpen(true)}
          className="hidden md:inline-flex"
        >
          <Search className="h-4 w-4" />
          Search
          <kbd className="ml-2 rounded bg-black/30 px-1.5 text-[10px] text-white/60">
            ⌘K
          </kbd>
        </Button>
        <Button variant="ghost" size="icon" aria-label="Notifications">
          <Bell className="h-4 w-4" />
        </Button>
        <div className="flex h-9 w-9 items-center justify-center rounded-full bg-[var(--accent)] text-xs font-semibold text-[var(--accent-foreground)]">
          TF
        </div>
      </div>
    </header>
  );
}
