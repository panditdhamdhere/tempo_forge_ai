import Link from "next/link";
import { SiteHeader } from "@/components/landing/site-header";

const posts = [
  {
    slug: "why-tempoforge",
    title: "Why Tempo needs a Cursor-class forge",
    excerpt: "Payments L1s deserve AI tooling that understands TIP-20 fees and Tempo RPC.",
  },
  {
    slug: "auditing-tip20",
    title: "Auditing TIP-20 payment flows",
    excerpt: "Common pitfalls when stablecoin fee tokens meet upgradeable vaults.",
  },
];

export default function BlogPage() {
  return (
    <main>
      <SiteHeader />
      <section className="mx-auto max-w-3xl px-6 pb-24 pt-32">
        <h1 className="display text-4xl font-semibold">Blog</h1>
        <div className="mt-10 space-y-6">
          {posts.map((post) => (
            <Link
              key={post.slug}
              href={`/blog/${post.slug}`}
              className="glass block rounded-2xl p-6 transition hover:bg-white/5"
            >
              <h2 className="text-xl font-semibold">{post.title}</h2>
              <p className="mt-2 text-sm text-white/60">{post.excerpt}</p>
            </Link>
          ))}
        </div>
      </section>
    </main>
  );
}
