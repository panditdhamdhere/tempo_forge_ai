"use client";

import Link from "next/link";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";

export function Hero() {
  return (
    <section className="relative min-h-[100svh] overflow-hidden mesh">
      <div className="absolute inset-0 bg-[url('/hero-tempo.svg')] bg-cover bg-center opacity-70" />
      <div className="absolute inset-0 bg-gradient-to-b from-transparent via-[#071018]/35 to-[#071018]" />

      <div className="relative mx-auto flex min-h-[100svh] max-w-6xl flex-col justify-end px-6 pb-20 pt-32 md:justify-center md:pb-28">
        <motion.p
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="display mb-4 text-4xl font-semibold text-[var(--accent)] md:text-5xl"
        >
          TempoForge AI
        </motion.p>
        <motion.h1
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.08 }}
          className="display max-w-3xl text-4xl font-semibold leading-[1.05] text-white md:text-6xl"
        >
          Ship on Tempo at the speed of thought.
        </motion.h1>
        <motion.p
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.16 }}
          className="mt-5 max-w-xl text-lg text-white/70"
        >
          Generate contracts, audit risks, debug failed txs, and deploy across
          Tempo networks — Cursor + Alchemy + Tenderly + Vercel for Tempo
          developers.
        </motion.p>
        <motion.div
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, delay: 0.24 }}
          className="mt-8 flex flex-wrap gap-3"
        >
          <Button asChild size="lg">
            <Link href="/dashboard">Start building</Link>
          </Button>
          <Button asChild size="lg" variant="secondary">
            <Link href="/docs">Read the docs</Link>
          </Button>
        </motion.div>
      </div>
    </section>
  );
}
