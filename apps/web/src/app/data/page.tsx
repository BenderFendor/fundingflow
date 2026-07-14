import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { DataBadge, DocumentHero, DocumentSection } from "@/components/ui";

const pipelineSteps = [
  ["01", "source_record", "Raw API response or imported file with source URL, retrieval time, content hash, and raw storage path."],
  ["02", "evidence", "Citation unit pointing to a field path or quotation inside a source record, with confidence metadata."],
  ["03", "entity", "Canonical organization, person, committee, or agency created from sourced records and normalized names."],
  ["04", "entity_identifier", "External identifiers such as UEI, CIK, FEC ID, or LDA ID attached to canonical entities."],
  ["05", "entity_alias", "Source-specific name variants retained for matching and review."],
  ["06", "identity_match", "Candidate or accepted cross-source match with method, confidence, and reviewer state."],
  ["07", "relationship_edge", "Neutral, typed connection between entities with amount, dates, source, and evidence link."],
];

const domainTables = [
  ["award", "USAspending federal award records with obligation and outlay amounts."],
  ["lobbying_filing", "LDA quarterly filings with registrants, clients, issues, and reported amounts."],
  ["campaign_finance_transaction", "FEC contribution records with committee, contributor, employer, and occupation fields."],
  ["sec_fact", "SEC XBRL facts with taxonomy, reporting period, unit, and filing accession."],
  ["rulemaking_record", "Federal Register and Regulations.gov documents, dockets, agencies, and comments."],
  ["metric_observation", "Source-backed economic values preserving geography, date, release date, and vintage."],
  ["derived_metric_observation", "Computed pressure indicators stored with formulas and named input metrics."],
];

const sources = ["USAspending", "LDA", "FEC", "SEC EDGAR", "Federal Register", "BLS", "DOL", "BEA", "Census", "FHFA", "HUD", "EIA"];

export default function DataPage() {
  return (
    <div className="min-h-screen text-white">
      <SiteHeader />
      <main id="main-content" className="mx-auto w-full max-w-6xl px-4 pb-16 pt-5 sm:px-6 lg:px-8">
        <DocumentHero
          eyebrow="Source chain"
          title="Data lineage"
          description="Every published record should be traceable from its public source through evidence, identity resolution, normalized domain data, and the relationship graph."
          index="01"
        />

        <div className="mt-5 grid gap-5">
          <DocumentSection index="01" title="Ingestion pipeline">
            <div className="divide-y divide-white/10 border-y border-white/10">
              {pipelineSteps.map(([index, table, description]) => (
                <div key={table} className="grid gap-3 py-5 sm:grid-cols-[3rem_13rem_1fr] sm:items-start">
                  <span className="ff-pixel text-lg font-black text-white/20">{index}</span>
                  <code className="break-all font-mono text-xs font-bold text-[#dfff00]">{table}</code>
                  <p>{description}</p>
                </div>
              ))}
            </div>
          </DocumentSection>

          <DocumentSection index="02" title="Domain tables">
            <div className="grid border-l border-t border-white/10 md:grid-cols-2">
              {domainTables.map(([table, description]) => (
                <article key={table} className="border-r border-b border-white/10 p-5">
                  <code className="font-mono text-xs font-bold text-white">{table}</code>
                  <p className="mt-3 text-sm leading-6 text-slate-500">{description}</p>
                </article>
              ))}
            </div>
          </DocumentSection>

          <DocumentSection index="03" title="Primary source families" tone="blueprint">
            <p className="max-w-3xl text-white/75">
              FundingFlow uses original public-agency sources first. Optional normalized datasets may enrich discovery, but they do not replace the canonical record or evidence link.
            </p>
            <div className="mt-7 flex flex-wrap gap-2">
              {sources.map((source) => <DataBadge key={source} tone="blue">{source}</DataBadge>)}
            </div>
          </DocumentSection>

          <DocumentSection index="04" title="Publication rule" tone="paper">
            <p>
              Missing data remains missing. A null observation is not converted to zero, a national price is not labeled as state-level data, and a federal obligation is not described as company revenue without separate accounting evidence.
            </p>
          </DocumentSection>
        </div>
      </main>
      <SiteFooter />
    </div>
  );
}
