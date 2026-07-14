import Link from "next/link";

const primaryLinks = [
  { href: "/", label: "Search" },
  { href: "/cost-basket", label: "Cost basket" },
  { href: "/data", label: "Data lineage" },
  { href: "/methodology", label: "Methodology" },
];

const stateLinks = ["PA", "CA", "TX", "NY"];

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40 border-b border-white/10 bg-[#050607]/88 backdrop-blur-xl">
      <div className="mx-auto flex w-full max-w-[1440px] items-center justify-between gap-5 px-4 py-3 sm:px-6 lg:px-8">
        <Link href="/" className="ff-focus-ring group flex shrink-0 items-center gap-3">
          <span className="grid h-9 w-9 place-items-center border border-[#dfff00]/35 bg-[#dfff00] font-mono text-[11px] font-black text-black shadow-[3px_3px_0_rgba(223,255,0,0.16)] transition-transform group-hover:-translate-y-0.5">
            FF
          </span>
          <span className="hidden sm:block">
            <span className="ff-sans-wide block text-lg font-black leading-none text-white">FundingFlow</span>
            <span className="mt-1 block font-mono text-[8px] uppercase tracking-[0.18em] text-slate-600">Public records intelligence</span>
          </span>
        </Link>

        <nav aria-label="Primary navigation" className="min-w-0 flex-1 overflow-x-auto">
          <div className="flex min-w-max items-center justify-center gap-1">
            {primaryLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="ff-focus-ring ff-fluid border border-transparent px-3 py-2 font-mono text-[9px] font-bold uppercase tracking-[0.13em] text-slate-400 hover:border-white/10 hover:bg-white/[0.045] hover:text-white"
              >
                {link.label}
              </Link>
            ))}
          </div>
        </nav>

        <div className="hidden shrink-0 items-center gap-1 lg:flex" aria-label="Featured state profiles">
          <span className="mr-2 font-mono text-[8px] uppercase tracking-[0.16em] text-slate-700">State deck</span>
          {stateLinks.map((state) => (
            <Link
              key={state}
              href={`/states/${state}`}
              className="ff-focus-ring ff-fluid border border-white/10 px-2.5 py-2 font-mono text-[9px] font-black text-slate-500 hover:border-[#dfff00]/30 hover:text-[#dfff00]"
            >
              {state}
            </Link>
          ))}
        </div>
      </div>
    </header>
  );
}
