import Link from "next/link";
import { SignIn } from "@clerk/nextjs";
import { SiteHeader } from "@/components/landing/site-header";
import { Button } from "@/components/ui/button";

export default function SignInPage() {
  const clerkEnabled = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

  return (
    <main>
      <SiteHeader />
      <section className="mx-auto flex min-h-[80vh] max-w-lg flex-col items-center justify-center px-6 pt-24">
        {clerkEnabled ? (
          <SignIn
            appearance={{
              elements: {
                rootBox: "mx-auto",
                card: "bg-[#0c1816] border border-white/10 shadow-none",
              },
            }}
            fallbackRedirectUrl="/dashboard"
            signUpUrl="/sign-up"
          />
        ) : (
          <div className="glass w-full rounded-3xl p-8">
            <h1 className="display text-3xl font-semibold">Sign in</h1>
            <p className="mt-3 text-sm text-white/65">
              Clerk keys are not configured. Local development continues with
              the dashboard using a development bearer token.
            </p>
            <Button asChild className="mt-8 w-full">
              <Link href="/dashboard">Continue to dashboard</Link>
            </Button>
          </div>
        )}
      </section>
    </main>
  );
}
