"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { Command } from "cmdk";
import { useUiStore } from "@/store/ui";

const items = [
  { label: "Overview", href: "/dashboard" },
  { label: "Projects", href: "/dashboard/projects" },
  { label: "AI Assistant", href: "/dashboard/assistant" },
  { label: "Auditor", href: "/dashboard/auditor" },
  { label: "Explorer", href: "/dashboard/explorer" },
  { label: "Analytics", href: "/dashboard/analytics" },
  { label: "Billing", href: "/dashboard/billing" },
  { label: "Settings", href: "/dashboard/settings" },
];

export function CommandPalette() {
  const open = useUiStore((s) => s.commandOpen);
  const setOpen = useUiStore((s) => s.setCommandOpen);
  const router = useRouter();

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen(!open);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, setOpen]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/60 p-4 backdrop-blur-sm">
      <Command
        className="glass mx-auto mt-[15vh] max-w-xl overflow-hidden rounded-2xl"
        label="Global command palette"
      >
        <Command.Input
          placeholder="Search TempoForge…"
          className="w-full border-b border-white/10 bg-transparent px-4 py-4 text-base outline-none"
          autoFocus
        />
        <Command.List className="max-h-80 overflow-auto p-2">
          <Command.Empty className="px-3 py-6 text-sm text-white/50">
            No results.
          </Command.Empty>
          {items.map((item) => (
            <Command.Item
              key={item.href}
              value={item.label}
              className="cursor-pointer rounded-xl px-3 py-2.5 text-sm aria-selected:bg-white/10"
              onSelect={() => {
                setOpen(false);
                router.push(item.href);
              }}
            >
              {item.label}
            </Command.Item>
          ))}
        </Command.List>
      </Command>
      <button
        className="absolute inset-0 -z-10"
        aria-label="Close command palette"
        onClick={() => setOpen(false)}
      />
    </div>
  );
}
