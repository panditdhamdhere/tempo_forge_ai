"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Activity,
  Bot,
  Boxes,
  FileSearch,
  FolderKanban,
  LayoutDashboard,
  Rocket,
  Settings,
  Shield,
  Wallet,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/store/ui";

const links = [
  { href: "/dashboard", label: "Overview", icon: LayoutDashboard },
  { href: "/dashboard/projects", label: "Projects", icon: FolderKanban },
  { href: "/dashboard/contracts", label: "Contracts", icon: Boxes },
  { href: "/dashboard/auditor", label: "Auditor", icon: Shield },
  { href: "/dashboard/assistant", label: "AI Assistant", icon: Bot },
  { href: "/dashboard/deployments", label: "Deployments", icon: Rocket },
  { href: "/dashboard/explorer", label: "Explorer", icon: FileSearch },
  { href: "/dashboard/analytics", label: "Analytics", icon: Activity },
  { href: "/dashboard/billing", label: "Billing", icon: Wallet },
  { href: "/dashboard/settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const pathname = usePathname();
  const collapsed = useUiStore((s) => s.sidebarCollapsed);

  return (
    <aside
      className={cn(
        "glass sticky top-0 flex h-screen flex-col border-r border-white/10 px-3 py-5 transition-all",
        collapsed ? "w-[76px]" : "w-64",
      )}
    >
      <Link href="/" className="display px-2 text-lg font-semibold">
        {collapsed ? "TF" : "TempoForge"}
      </Link>
      <nav className="mt-8 flex flex-1 flex-col gap-1">
        {links.map(({ href, label, icon: Icon }) => {
          const active =
            pathname === href ||
            (href !== "/dashboard" && pathname.startsWith(href));
          return (
            <Link
              key={href}
              href={href}
              className={cn(
                "flex items-center gap-3 rounded-xl px-3 py-2.5 text-sm text-white/65 transition hover:bg-white/5 hover:text-white",
                active && "bg-white/10 text-white",
              )}
            >
              <Icon className="h-4 w-4 shrink-0" />
              {!collapsed && <span>{label}</span>}
            </Link>
          );
        })}
      </nav>
    </aside>
  );
}
