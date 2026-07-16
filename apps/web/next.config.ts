import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  transpilePackages: ["@tempoforge/types", "@tempoforge/prompts", "@tempoforge/config"],
};

export default nextConfig;
