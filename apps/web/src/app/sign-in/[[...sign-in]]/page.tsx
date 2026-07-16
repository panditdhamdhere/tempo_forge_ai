import Link from "next/link";
import { SiteHeader } from "@/components/landing/site-header";
import { Button } from "@/components/ui/button";

export default function SignInPage() {
  const clerkEnabled = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

  return (
    <main>
      <SiteHeader />
      <section className="mx-auto flex min-h-[80vh] max-w-lg flex-col justify-center px-6 pt-24">
        <div className="glass rounded-3xl p-8">
          <h1 className="display text-3xl font-semibold">Sign in</h1>
          <p className="mt-3 text-sm text-white/65">
            {clerkEnabled
              ? "Clerk is configured. Use your organization credentials."
              : "Development mode: Clerk keys are not set. Continue with the local dashboard."}
          </p>
          <Button asChild className="mt-8 w-full">
            <Link href="/dashboard">Continue to dashboard</Link>
          </Button>
        </div>
      </section>
    </main>
  );
}
