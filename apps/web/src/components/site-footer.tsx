import Link from "next/link";

export function SiteFooter() {
  return (
    <footer className="border-t border-white/10 bg-black/30">
      <div className="mx-auto grid w-full max-w-[1440px] gap-8 px-4 py-10 sm:px-6 md:grid-cols-[1.4fr_1fr_1fr] lg:px-8">
        <div>
          <p className="ff-sans-wide text-xl font-black text-white">FundingFlow</p>
          <p className="mt-3 max-w-md text-sm leading-6 text-slate-500">
            A source-backed workspace for tracing federal obligations, lobbying records, campaign finance, corporate disclosures, and economic conditions.
          </p>
        </div>
        <div>
          <p className="ff-micro text-[9px] font-black text-slate-600">Documentation</p>
          <div className="mt-4 flex flex-col items-start gap-2 font-mono text-[10px] uppercase tracking-wider text-slate-400">
            <Link className="ff-focus-ring hover:text-[#dfff00]" href="/data">Data lineage</Link>
            <Link className="ff-focus-ring hover:text-[#dfff00]" href="/methodology">Methodology</Link>
            <Link className="ff-focus-ring hover:text-[#dfff00]" href="/cost-basket">Cost basket</Link>
          </div>
        </div>
        <div>
          <p className="ff-micro text-[9px] font-black text-slate-600">Interpretation rule</p>
          <p className="mt-4 text-xs leading-5 text-slate-600">
            Documented relationships do not prove causation, intent, policy influence, or wrongdoing.
          </p>
        </div>
      </div>
    </footer>
  );
}
