import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { DataBadge, DocumentHero, DocumentSection } from "@/components/ui";

const sources = [
  ["USAspending.gov", "Federal awards, recipients, agencies, dates, and obligation amounts."],
  ["LDA.gov", "Lobbying filings, registrants, clients, lobbyists, issues, and reported amounts."],
  ["FEC.gov", "Campaign committees, candidates, receipts, disbursements, and individual contributions."],
  ["SEC EDGAR", "CIKs, filings, XBRL facts, legal names, and corporate disclosure records."],
  ["Federal Register", "Rules, proposed rules, notices, agencies, and comment periods."],
  ["Economic agencies", "BLS, DOL, BEA, Census, FHFA, HUD, and EIA observations."],
];

const limitations = [
  "Employer names in campaign-finance records are user-entered and treated as weak identity evidence.",
  "Federal obligations are not recognized revenue and require separate period alignment before comparison with SEC facts.",
  "Name-based entity matches without strong identifiers retain visible confidence and review state.",
  "Missing observations are not converted to zero or silently filled with unrelated geography.",
  "National or regional food prices are never presented as state grocery estimates.",
];

export default function MethodologyPage() {
  return (
    <div className="min-h-screen text-white">
      <SiteHeader />
      <main id="main-content" className="mx-auto w-full max-w-6xl px-4 pb-16 pt-5 sm:px-6 lg:px-8">
        <DocumentHero
          eyebrow="Evidence rules"
          title="Methodology"
          description="FundingFlow stores public records with provenance, resolves identities in stages, and publishes neutral relationships without turning correlation into an accusation."
          index="02"
        />

        <div className="mt-5 grid gap-5">
          <DocumentSection index="01" title="How the system works">
            <p>
              Source-specific importers acquire public records, preserve raw retrieval metadata, normalize domain records, and attach evidence to identifiers, metrics, awards, and relationship edges. The public interface reads from those normalized records rather than from unsourced summaries.
            </p>
          </DocumentSection>

          <DocumentSection index="02" title="Entity resolution">
            <p>
              Matching starts with deterministic identifiers such as UEI, CIK, FEC committee ID, and LDA registrant or client ID. Exact normalized names are considered next, followed by PostgreSQL trigram candidates. Weak or ambiguous matches should remain reviewable instead of being merged automatically.
            </p>
            <div className="mt-7 flex flex-wrap gap-2">
              <DataBadge tone="acid">Identifiers first</DataBadge>
              <DataBadge>Exact names second</DataBadge>
              <DataBadge>Fuzzy candidates reviewed</DataBadge>
            </div>
          </DocumentSection>

          <DocumentSection index="03" title="Evidence and provenance" tone="blueprint">
            <p className="text-white/75">
              Each relationship edge and normalized record links to an evidence row. Evidence points to a source record, URL, retrieval timestamp, field path or quotation, and confidence metadata. This keeps a visible chain between the public claim and the underlying document.
            </p>
          </DocumentSection>

          <DocumentSection index="04" title="Neutral relationship language">
            <p>
              Edge labels describe what the record documents. An agency can obligate an award to a recipient; a registrant can report lobbying for a client; a committee can report receiving a contribution. These labels do not, by themselves, establish intent, influence, favoritism, or wrongdoing.
            </p>
          </DocumentSection>

          <DocumentSection index="05" title="Primary data sources">
            <div className="divide-y divide-white/10 border-y border-white/10">
              {sources.map(([source, description], index) => (
                <div key={source} className="grid gap-3 py-5 sm:grid-cols-[3rem_12rem_1fr]">
                  <span className="ff-pixel text-lg font-black text-white/20">{String(index + 1).padStart(2, "0")}</span>
                  <strong className="text-sm text-white">{source}</strong>
                  <p>{description}</p>
                </div>
              ))}
            </div>
          </DocumentSection>

          <DocumentSection index="06" title="Economic conditions layer" tone="paper">
            <p>
              Economic observations preserve source, geography, date, release date, and vintage date. FundingFlow calculations are stored separately with formulas and input metric IDs so readers can distinguish an agency observation from a derived pressure indicator.
            </p>
          </DocumentSection>

          <DocumentSection index="07" title="Known limitations">
            <ol className="divide-y divide-white/10 border-y border-white/10">
              {limitations.map((limitation, index) => (
                <li key={limitation} className="grid gap-3 py-5 sm:grid-cols-[3rem_1fr]">
                  <span className="ff-pixel text-lg font-black text-[#ee744f]">{String(index + 1).padStart(2, "0")}</span>
                  <p>{limitation}</p>
                </li>
              ))}
            </ol>
          </DocumentSection>
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
