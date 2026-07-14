import Link from "next/link";

export default function DataPage() {
  return (
    <main className="mx-auto min-h-screen max-w-4xl px-6 py-12 text-slate-100">
      <Link href="/" className="ff-focus-ring ff-micro text-sm font-black text-slate-500 hover:text-slate-200">
        &larr; Back to search
      </Link>

      <section className="ff-paper ff-heavy-border mt-6 rounded-[1.75rem] p-6 text-black sm:p-8">
        <p className="ff-micro text-xs font-black text-black/50">Source chain</p>
        <h1 className="mt-3 text-6xl font-black leading-[0.9] tracking-[-0.08em]">Data Lineage</h1>

        <p className="mt-4 max-w-2xl text-lg font-semibold text-black/65">
          Every record in FundingFlow traces back to its public source. This page
          documents the data flow from source to graph.
        </p>
      </section>

      <section className="ff-terminal mt-10 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Ingestion Pipeline</h2>
        <div className="mt-4 overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-white/15">
                <th className="pb-2 font-semibold text-slate-300">Step</th>
                <th className="pb-2 font-semibold text-slate-300">Table</th>
                <th className="pb-2 font-semibold text-slate-300">Description</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/10">
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">1</td>
                <td className="py-2 font-mono text-sm">source_record</td>
                <td className="py-2 text-slate-400">
                  Raw API response or imported file. Stores source URL,
                  retrieval timestamp, content hash, and raw storage path.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">2</td>
                <td className="py-2 font-mono text-sm">evidence</td>
                <td className="py-2 text-slate-400">
                  Citation unit that points to a specific field path or data
                  quote within a source record. Carries a confidence score.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">3</td>
                <td className="py-2 font-mono text-sm">entity</td>
                <td className="py-2 text-slate-400">
                  Canonical organization, person, committee, or agency.
                  Created from source records with entity type and
                  normalized name.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">4</td>
                <td className="py-2 font-mono text-sm">entity_identifier</td>
                <td className="py-2 text-slate-400">
                  External identifier (UEI, CIK, FEC ID, LDA ID) attached
                  to a canonical entity. Links back to source evidence.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">5</td>
                <td className="py-2 font-mono text-sm">entity_alias</td>
                <td className="py-2 text-slate-400">
                  Name variant for an entity from a specific source.
                  Normalized for matching.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">6</td>
                <td className="py-2 font-mono text-sm">identity_match</td>
                <td className="py-2 text-slate-400">
                  Candidate or accepted match between two source records or
                  entities. Includes method, confidence, and reviewer info.
                </td>
              </tr>
              <tr>
                <td className="ff-mono-num py-2 text-slate-500">7</td>
                <td className="py-2 font-mono text-sm">relationship_edge</td>
                <td className="py-2 text-slate-400">
                  Typed edge between two entities. Includes amount, date
                  range, edge type, source, and evidence link.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Domain Tables</h2>
        <ul className="mt-4 space-y-3 text-slate-400">
          <li>
            <strong className="font-mono text-sm">award</strong> &mdash;
            USAspending federal award records with obligation and outlay amounts.
          </li>
          <li>
            <strong className="font-mono text-sm">lobbying_filing</strong> &mdash;
            LDA quarterly lobbying disclosure filings with issue codes.
          </li>
          <li>
            <strong className="font-mono text-sm">campaign_finance_transaction</strong> &mdash;
            FEC contribution records with employer text and occupation.
          </li>
          <li>
            <strong className="font-mono text-sm">sec_fact</strong> &mdash;
            SEC XBRL company facts including revenue with taxonomy and period.
          </li>
          <li>
            <strong className="font-mono text-sm">rulemaking_record</strong> &mdash;
            Federal Register and Regulations.gov documents and comments.
          </li>
          <li>
            <strong className="font-mono text-sm">geo</strong> &mdash;
            Nation, state, county, and metro geography identifiers used to connect
            public money to economic conditions.
          </li>
          <li>
            <strong className="font-mono text-sm">metric_observation</strong> &mdash;
            Source-backed economic observations with date, release date, and
            vintage date for labor, wages, housing, food, energy, income, and
            public-money metrics.
          </li>
          <li>
            <strong className="font-mono text-sm">derived_metric_observation</strong> &mdash;
            Computed pressure indicators such as rent hours, contract intensity,
            and housing-wage squeeze with formula metadata.
          </li>
        </ul>
      </section>

      <section className="ff-blueprint mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Economic Sources</h2>
        <p className="mt-3 text-[#f3f0d9]/75">
          The State MVP uses original agency sources first: BLS, DOL, BEA,
          Census ACS, FHFA, HUD, EIA, and USAspending. Food item prices are
          labeled as U.S. or broad regional averages unless a local public
          source supports narrower geography.
        </p>
      </section>

      <p className="mt-16 text-xs text-gray-400">
        FundingFlow shows relationships documented in public records. These
        links do not prove causation, intent, policy influence, or wrongdoing.
      </p>
    </main>
  );
}
