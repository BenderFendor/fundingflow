import Link from "next/link";

export default function MethodologyPage() {
  return (
    <main className="mx-auto min-h-screen max-w-4xl px-6 py-12 text-slate-100">
      <Link href="/" className="ff-focus-ring ff-micro text-sm font-black text-slate-500 hover:text-slate-200">
        &larr; Back to search
      </Link>

      <section className="ff-paper ff-heavy-border mt-6 rounded-[1.75rem] p-6 text-black sm:p-8">
        <p className="ff-micro text-xs font-black text-black/50">Evidence rules</p>
        <h1 className="mt-3 text-6xl font-black leading-[0.9] tracking-[-0.08em]">Methodology</h1>
        <p className="mt-4 max-w-2xl text-lg font-semibold text-black/65">
          Public records are stored with source records, evidence rows, neutral edges, and visible limitations.
        </p>
      </section>

      <section className="ff-terminal mt-8 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">How FundingFlow Works</h2>
        <p className="mt-3 text-slate-400">
          FundingFlow ingests public records from federal data sources, creates
          canonical entities for organizations, people, committees, and
          agencies, then links them through source-backed relationship edges.
        </p>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Entity Resolution</h2>
        <p className="mt-3 text-slate-400">
          Entities are matched across data sources using deterministic
          identifiers first (UEI, CIK, FEC committee ID, LDA registrant/client
          ID), then exact normalized-name matching, then PostgreSQL pg_trgm
          fuzzy similarity search. Matches with lower confidence scores or
          ambiguous evidence are flagged for review.
        </p>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Evidence And Provenance</h2>
        <p className="mt-3 text-slate-400">
          Every relationship edge, entity identifier, metric, and award record
          links back to a source record and evidence row. Source records store
          the original API response, retrieval timestamp, content hash, and
          source URL. Evidence rows cite the specific field path or data quote
          used.
        </p>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Edge Types</h2>
        <p className="mt-3 text-slate-400">
          Relationship edges use neutral labels that describe the documented
          relationship without inferring causation or intent. For example,
          money moving from an agency to a recipient is labeled
          &ldquo;agency obligated award to recipient&rdquo; and
          &ldquo;recipient received federal obligation,&rdquo; not
          &ldquo;contract revenue&rdquo; unless that fact is independently
          reported.
        </p>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Data Sources</h2>
        <ul className="mt-3 space-y-3 text-slate-400">
          <li>
            <strong>USAspending.gov</strong> &mdash; Federal awards, recipients,
            agencies, award dates, and obligation amounts. Bulk awards API with
            no required authentication.
          </li>
          <li>
            <strong>LDA.gov</strong> &mdash; Lobbying Disclosure Act filings,
            including registrants, clients, lobbyists, issue areas, and
            quarterly income/expense amounts.
          </li>
          <li>
            <strong>FEC.gov</strong> &mdash; Campaign finance data including
            committee receipts, individual contributions, employer text, and
            lobbyist bundled contributions. Bulk file import.
          </li>
          <li>
            <strong>SEC EDGAR</strong> &mdash; Company identifiers (CIK),
            tickers, registration data, and XBRL-tagged facts including
            reported revenue figures.
          </li>
          <li>
            <strong>Federal Register</strong> &mdash; Rulemaking documents,
            proposed and final rules, agency assignments, and comment periods.
          </li>
          <li>
            <strong>BLS, DOL, BEA, Census, FHFA, HUD, and EIA</strong> &mdash;
            Labor markets, wages, minimum wage law, state economic accounts,
            housing, food and energy pressure, and public-money context.
          </li>
        </ul>
      </section>

      <section className="ff-blueprint mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Economic Conditions Layer</h2>
        <p className="mt-3 text-[#f3f0d9]/75">
          Economic observations preserve source, date, release date, and vintage
          date. Derived indicators are stored separately with their formulas and
          input metrics so users can distinguish agency data from FundingFlow
          calculations.
        </p>
      </section>

      <section className="ff-terminal mt-6 rounded-[1.5rem] p-6">
        <h2 className="text-xl font-semibold">Limitations</h2>
        <ul className="mt-3 list-disc space-y-2 pl-6 text-slate-400">
          <li>
            Employer names in campaign finance records are user-entered text.
            They are treated as weak evidence and linked with lower confidence.
          </li>
          <li>
            Obligated amounts are not recognized revenue. Comparisons between
            federal obligations and SEC reported revenue require careful date
            alignment and separate source citations.
          </li>
          <li>
            Entity matching across sources relies on name similarity and
            available identifiers. Matches without strong identifiers carry
            visible confidence scores.
          </li>
          <li>
            Missing data is shown as missing. Empty aggregate values are not
            converted to zero.
          </li>
          <li>
            Regional or national food prices are not presented as state-level
            grocery prices.
          </li>
        </ul>
      </section>

      <p className="mt-16 text-xs text-gray-400">
        FundingFlow shows relationships documented in public records. These
        links do not prove causation, intent, policy influence, or wrongdoing.
      </p>
    </main>
  );
}
