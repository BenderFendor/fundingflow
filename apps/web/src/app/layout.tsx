import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "FundingFlow | Federal Intelligence",
  description:
    "Political-economy data platform for public money, private power, and household pressure.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="bg-[#0a0a0a] text-slate-200 antialiased font-sans min-h-screen selection:bg-emerald-500/30">
        {children}
      </body>
    </html>
  );
}
