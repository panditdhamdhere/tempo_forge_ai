"use client";

import { useCallback, useEffect, useState } from "react";

type ClerkSessionLike = {
  getToken: () => Promise<string | null>;
};

type ClerkLike = {
  session?: ClerkSessionLike | null;
  loaded?: boolean;
  user?: unknown;
};

function readClerk(): ClerkLike | null {
  if (typeof window === "undefined") return null;
  return (window as unknown as { Clerk?: ClerkLike }).Clerk ?? null;
}

/**
 * Bearer token helper that works with or without Clerk.
 * When Clerk is loaded on window, uses the session JWT; otherwise `dev`.
 */
export function useApiToken() {
  const [ready, setReady] = useState(
    () => !process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY,
  );
  const [signedIn, setSignedIn] = useState(
    () => !process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY,
  );

  useEffect(() => {
    if (!process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY) {
      setReady(true);
      setSignedIn(true);
      return;
    }

    let cancelled = false;
    const tick = () => {
      const clerk = readClerk();
      if (!clerk) return;
      if (!cancelled) {
        setReady(Boolean(clerk.loaded ?? true));
        setSignedIn(Boolean(clerk.session));
      }
    };

    tick();
    const id = window.setInterval(tick, 400);
    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, []);

  const getToken = useCallback(async () => {
    const clerk = readClerk();
    if (clerk?.session) {
      const token = await clerk.session.getToken();
      if (token) return token;
    }
    return "dev";
  }, []);

  return {
    getToken,
    isLoaded: ready,
    isSignedIn: signedIn,
  };
}
