import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "FundingFlow | Federal Intelligence",
  description:
    "Political-economy data platform for public money, private power, and household pressure.",
  icons: {
    icon: "/icon.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="ff-shell bg-[#050505] text-slate-200 antialiased font-sans min-h-screen">
        {children}
      </body>
    </html>
  );
}
