import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "FundingFlow | Public Records Intelligence",
    template: "%s | FundingFlow",
  },
  description:
    "Trace federal spending, lobbying, campaign finance, corporate disclosures, and economic conditions through source-backed public records.",
  icons: {
    icon: "/icon.svg",
  },
};

export const viewport: Viewport = {
  colorScheme: "dark",
  themeColor: "#050607",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="ff-shell min-h-screen bg-[#050607] text-slate-200 antialiased">
        <a
          href="#main-content"
          className="ff-focus-ring fixed left-4 top-4 z-[80] -translate-y-24 bg-[#dfff00] px-4 py-2 font-mono text-[10px] font-black uppercase tracking-wider text-black transition-transform focus:translate-y-0"
        >
          Skip to content
        </a>
        {children}
      </body>
    </html>
  );
}
