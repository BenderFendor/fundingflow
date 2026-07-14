import Link from "next/link";
import { getStateProfile } from "@/lib/types";

type CountyPageProps = {
  params: Promise<{ stateCode: string; countyCode: string }>;
};

export default async function CountyProfilePage({ params }: CountyPageProps) {
  const { stateCode, countyCode } = await params;

  let stateProfile = null;
  try {
    stateProfile = await getStateProfile(stateCode.toUpperCase());
  } catch {
    // API not running
  }

  return (
    <main className="mx-auto min-h-screen max-w-4xl px-4 py-8 text-slate-100">
      <header className="ff-paper ff-heavy-border mb-8 rounded-[1.75rem] p-6 text-black sm:p-8">
        <Link href="/" className="ff-focus-ring ff-micro text-sm font-black text-black/45 hover:text-black">
          &larr; Back to search
        </Link>
        {stateProfile && (
          <Link
            href={`/states/${stateCode.toUpperCase()}`}
            className="ff-focus-ring ff-micro ml-4 text-sm font-black text-black/45 hover:text-black"
          >
            &larr; {stateProfile.geo.name} profile
          </Link>
        )}
        <p className="ff-micro mt-8 text-sm font-black text-black/50">County</p>
        <h1 className="mt-2 text-6xl font-black leading-[0.9] tracking-[-0.08em]">
          {countyCode}, {stateCode.toUpperCase()}
        </h1>
        <p className="mt-4 max-w-2xl text-lg font-semibold text-black/65">
          County-level economic data is not yet imported. This page will populate as
          county data sources are added to the ingestion pipeline.
        </p>
      </header>

      <div className="ff-terminal rounded-xl border border-dashed border-white/10 p-10 text-center">
        <p className="text-2xl font-bold text-slate-500 mb-4">Awaiting county data</p>
        <p className="text-sm text-slate-500 max-w-lg mx-auto">
          The current ingestion pipeline covers state-level economic data from BLS,
          DOL, FHFA, Census ACS, and USAspending. County-level ingestion for Census ACS,
          BLS QCEW, IRS SOI, and other sources is planned but not yet implemented.
        </p>
        <p className="mt-6 text-xs text-slate-600">
          When county data is available, this page will show labor, housing, income,
          poverty, and public-money metrics for the county, with source-backed evidence
          and vintage labels.
        </p>
      </div>
    </main>
  );
}
