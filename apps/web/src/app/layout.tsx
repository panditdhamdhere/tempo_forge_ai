import type { Metadata } from "next";
import { IBM_Plex_Sans, Sora } from "next/font/google";
import { Providers } from "@/components/providers";
import "./globals.css";

const sora = Sora({
  subsets: ["latin"],
  variable: "--font-sora",
});

const plex = IBM_Plex_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-plex",
});

export const metadata: Metadata = {
  title: "TempoForge AI",
  description:
    "The AI-powered developer platform for Tempo Blockchain — generate, audit, debug, deploy, and explore.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${sora.variable} ${plex.variable} dark`}>
      <body
        className="min-h-screen antialiased"
        style={
          {
            "--font-display": "var(--font-sora)",
            "--font-body": "var(--font-plex)",
          } as React.CSSProperties
        }
      >
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
