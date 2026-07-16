import Link from "next/link";
import { SiteHeader } from "@/components/landing/site-header";
import { Button } from "@/components/ui/button";

export default function SignUpPage() {
  return (
    <main>
      <SiteHeader />
      <section className="mx-auto flex min-h-[80vh] max-w-lg flex-col justify-center px-6 pt-24">
        <div className="glass rounded-3xl p-8">
          <h1 className="display text-3xl font-semibold">Create account</h1>
          <p className="mt-3 text-sm text-white/65">
            Production uses Clerk. Locally you can enter the forge without an account.
          </p>
          <Button asChild className="mt-8 w-full">
            <Link href="/dashboard">Enter TempoForge</Link>
          </Button>
        </div>
      </section>
    </main>
  );
}
