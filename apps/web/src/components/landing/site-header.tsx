import Link from "next/link";
import { Button } from "@/components/ui/button";

export function SiteHeader() {
  return (
    <header className="absolute inset-x-0 top-0 z-20">
      <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-6">
        <Link href="/" className="display text-lg font-semibold tracking-tight">
          TempoForge<span className="text-[var(--accent)]"> AI</span>
        </Link>
        <nav className="hidden items-center gap-8 text-sm text-white/70 md:flex">
          <Link href="/#features" className="hover:text-white">
            Features
          </Link>
          <Link href="/pricing" className="hover:text-white">
            Pricing
          </Link>
          <Link href="/docs" className="hover:text-white">
            Docs
          </Link>
          <Link href="/blog" className="hover:text-white">
            Blog
          </Link>
        </nav>
        <div className="flex items-center gap-3">
          <Button asChild variant="ghost" size="sm">
            <Link href="/sign-in">Sign in</Link>
          </Button>
          <Button asChild size="sm">
            <Link href="/dashboard">Open forge</Link>
          </Button>
        </div>
      </div>
    </header>
  );
}
