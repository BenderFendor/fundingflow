import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Space Grotesk"', 'Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['"Azeret Mono"', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'monospace'],
        display: ['"Space Grotesk"', 'sans-serif'],
        pixel: ['"Azeret Mono"', 'monospace'],
      },
      colors: {
        gray: {
          850: "#1f2937",
          900: "#111827",
          950: "#030712",
        },
        bento: {
          purple: "#A599D0",
          yellow: "#FFDD00",
          olive: "#A4A626",
          orange: "#FF6B3B",
          blue: "#C6E2E9",
          cardbg: "#1C1C1C",
          dark: "#0a0a0a"
        }
      }
    },
  },
  plugins: [],
};

export default config;
