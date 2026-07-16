import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["@tempoforge/types", "@tempoforge/prompts", "@tempoforge/config"],
};

export default nextConfig;
