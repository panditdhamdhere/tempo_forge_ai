import Link from "next/link";
import { SignUp } from "@clerk/nextjs";
import { SiteHeader } from "@/components/landing/site-header";
import { Button } from "@/components/ui/button";

export default function SignUpPage() {
  const clerkEnabled = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

  return (
    <main>
      <SiteHeader />
      <section className="mx-auto flex min-h-[80vh] max-w-lg flex-col items-center justify-center px-6 pt-24">
        {clerkEnabled ? (
          <SignUp
            appearance={{
              elements: {
                rootBox: "mx-auto",
                card: "bg-[#0c1816] border border-white/10 shadow-none",
              },
            }}
            fallbackRedirectUrl="/dashboard"
            signInUrl="/sign-in"
          />
        ) : (
          <div className="glass w-full rounded-3xl p-8">
            <h1 className="display text-3xl font-semibold">Create account</h1>
            <p className="mt-3 text-sm text-white/65">
              Production uses Clerk. Locally you can enter the forge without an
              account.
            </p>
            <Button asChild className="mt-8 w-full">
              <Link href="/dashboard">Enter TempoForge</Link>
            </Button>
          </div>
        )}
      </section>
    </main>
  );
}
