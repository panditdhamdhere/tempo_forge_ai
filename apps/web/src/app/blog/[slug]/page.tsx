import { notFound } from "next/navigation";
import { SiteHeader } from "@/components/landing/site-header";

const posts: Record<string, { title: string; body: string }> = {
  "why-tempoforge": {
    title: "Why Tempo needs a Cursor-class forge",
    body: "Tempo’s payment-first design changes how developers think about balances, fees, and tooling. TempoForge AI packs generation, auditing, debugging, and exploration into one product surface so teams ship faster without losing security rigor.",
  },
  "auditing-tip20": {
    title: "Auditing TIP-20 payment flows",
    body: "When fees and balances live in TIP-20 tokens, classic eth_getBalance assumptions break. TempoForge’s auditor combines deterministic detectors with LLM reasoning so payment lanes, sponsorship, and vaults get reviewed in context.",
  },
};

export default async function BlogPostPage({
  params,
}: {
  params: Promise<{ slug: string }>;
}) {
  const { slug } = await params;
  const post = posts[slug];
  if (!post) notFound();

  return (
    <main>
      <SiteHeader />
      <article className="mx-auto max-w-3xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold">{post.title}</h1>
        <p className="mt-8 text-lg leading-relaxed text-white/75">{post.body}</p>
      </article>
    </main>
  );
}
