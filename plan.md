# FundingFlow Plan

FundingFlow is a public-record graph for federal spending, lobbying, campaign finance, corporate disclosures, and rulemaking records. The product should help users trace source-backed relationships between organizations, people, awards, filings, committees, and regulatory events.

The project should not claim causation, corruption, favoritism, or intent. It should show documented relationships, cite the records behind each edge, and make uncertainty visible.

## Product Frame

### Public Name

FundingFlow

### Internal Frame

State-Capital Graph

### One-Line Pitch

FundingFlow maps public records into a searchable graph of federal money, lobbying activity, political finance, company disclosures, and rulemaking participation.

### Primary Users

- Journalists tracing organizations across public records.
- Researchers studying federal spending and policy networks.
- Civic-tech users who need source-backed relationship maps.
- Recruiters or evaluators reviewing a data engineering and full-stack portfolio project.

### Non-Goals

- Do not infer corruption or quid pro quo.
- Do not rank entities as suspicious without a clear, explainable public-record basis.
- Do not treat campaign donations as proof of policy influence.
- Do not treat obligations as recognized revenue.
- Do not ship private dossiers. Use public records and clear citations.

## Language Rules

Use neutral relationship labels:

- `public_spending_relation`
- `contracting_relation`
- `lobbying_relation`
- `campaign_finance_relation`
- `disclosure_relation`
- `rulemaking_relation`
- `identity_match_relation`

Avoid claims like:

- `corruption`
- `favorable regulation`
- `influence buying`
- `pay to play`
- `dark money pipeline`

Use disclaimers near graph views and metrics:

> FundingFlow shows relationships documented in public records. These links do not prove causation, intent, policy influence, or wrongdoing.

For money metrics, use precise labels:

- Use `federal obligations` for USAspending award amounts.
- Use `federal obligations as share of reported revenue` for SEC comparison metrics.
- Do not label obligations as `contract revenue` unless a source supports revenue recognition.

## Source Matrix

| Source | Use | Access Pattern | Notes |
| --- | --- | --- | --- |
| USAspending.gov | Federal awards, recipients, agencies, award dates, award amounts | Public API and bulk downloads | No auth currently required for core API. Use bulk for backfills and API for lookup or deltas. |
| SAM.gov / GSA Entity APIs | Entity identifiers, registration data, exclusions, assistance listings, opportunities | Public extracts and selected APIs | Prefer public extracts for bulk ingestion. Use live APIs for spot checks and refresh flows. |
| LDA.gov | Lobbying registrations, quarterly filings, clients, registrants, lobbyists, issues, contribution reports | Public API at `https://lda.gov/api/v1/` | Use canonical `lda.gov`, not the older `lda.senate.gov` host. |
| FEC / OpenFEC | Committees, candidates, receipts, disbursements, individual contributions, lobbyist bundled contributions | Bulk files first, API when key is available | Direct API access can require signup or key handling. Bulk data is better for reproducible local builds. |
| SEC EDGAR | Company identifiers, filings, facts, revenue, subsidiaries, legal names | `data.sec.gov` REST APIs and filing archives | No API key. Requires fair access behavior. Use backend ingestion because CORS blocks browser-only access. |
| Federal Register | Rules, proposed rules, notices, agencies, docket references | JSON API | API endpoint works even when docs pages are gated. Good for rulemaking event search. |
| Regulations.gov | Dockets, documents, comments, agencies | GSA API with `X-Api-Key` | Use for deeper docket and comment records after API key setup. |
| LobbyView | Optional lobbying enrichment and normalized data | REST API and datasets | Treat as optional enrichment, not source of truth. |

## Verified API Notes

### USAspending.gov

Useful endpoints:

- `/api/v2/awards/<AWARD_ID>/`
- `/api/v2/download/*`
- `/api/v2/bulk_download/*`
- `/api/v2/search/new_awards_over_time/`

Implementation notes:

- Start with bulk award data for reproducible ingestion.
- Store raw response metadata and source URLs.
- Keep award identifiers, recipient UEI, recipient name, agency, dates, type, and obligated amount.

### LDA.gov

Canonical base URL:

- `https://lda.gov/api/v1/`

Observed resources:

- `filings`
- `contributions`
- `registrants`
- `clients`
- `lobbyists`
- constants endpoints

Observed counts during research:

- filings: `1,948,746`
- contributions: `656,646`
- clients: `135,326`
- registrants: `17,352`

Implementation notes:

- Model registrants, clients, lobbyists, and filing periods separately.
- Preserve issue codes and filing identifiers.
- Treat quarterly lobbying amounts as disclosed ranges or reported amounts according to the source fields.
- LDA API rate limits: 15 req/min unauthenticated, 120 req/min with API key. Respect these when paginating.
- `lda.senate.gov` will shut down after 2026-06-30. Use `lda.gov` as the canonical host.

### SAM.gov / GSA

Useful docs:

- `https://open.gsa.gov/api/entity-api/`
- `https://open.gsa.gov/api/sam-entity-extracts-api/`
- `https://open.gsa.gov/api/contract-awards/`
- `https://open.gsa.gov/api/get-opportunities-public-api/`
- `https://open.gsa.gov/api/fh-public-api/`

Implementation notes:

- Use SAM entity extracts as the main bulk source.
- Use UEI as a strong identifier when available.
- Store registration status, legal business name, DBA names, physical address, entity type, and exclusion status if available.

### FEC / OpenFEC

Useful sources:

- FEC bulk data pages for individual contributions.
- FEC bulk data pages for lobbyist bundled contributions.
- `https://github.com/fecgov/openFEC`

Implementation notes:

- Prefer bulk import for early MVP to avoid key friction.
- Model committees, candidates, contributors, employers, occupations, contribution dates, and amounts.
- Use employer-name matching carefully. Employer text is user-entered and noisy.
- Keep lobbyist bundled contribution records separate from individual contribution records.

### SEC EDGAR

Useful sources:

- `https://data.sec.gov/submissions/CIK##########.json`
- `https://data.sec.gov/api/xbrl/companyfacts/CIK##########.json`
- Filing archives for 10-K, 10-Q, 8-K, exhibits, and Exhibit 21 subsidiary lists.

Implementation notes:

- Use CIK as the primary SEC identifier. CIK must be zero-padded to 10 digits in API URLs.
- Use ticker and company title as lookup aids, not stable identifiers.
- Store reported revenue facts with taxonomy, period, form, frame, unit, and filing accession.
- SEC rate limit: 10 req/sec per IP. Add 100-120ms delay between requests.
- `User-Agent` header required with identifying contact information.
- Extract subsidiary evidence from Exhibit 21 only when needed for MVP stories.
- EDGAR Release 26.1 (March 2026) updated supported taxonomies and fee limits. No breaking API changes.

### Rulemaking

Useful sources:

- `https://www.federalregister.gov/api/v1/documents.json`
- Regulations.gov API through GSA with `X-Api-Key`.

Implementation notes:

- Use Federal Register for discovery of documents, agencies, rule types, and docket references.
- Use Regulations.gov for docket details, documents, and comments.
- Model `comment_submitted`, `document_published`, and `agency_issued_document` as separate edges.

## Open-Source Precedents

### USAspending API

Architecture precedent:

- PostgreSQL as the relational system of record.
- S3 or data lake style storage for bulk data.
- Elasticsearch or OpenSearch for search.
- Spark or batch ETL for large transforms.
- Materialized views and download jobs for heavy queries.

FundingFlow takeaway:

- Use relational truth plus search and graph read models. Do not start pure graph-first.

### OpenFEC

Architecture precedent:

- Flask and SQLAlchemy API layer.
- PostgreSQL with materialized views.
- Elasticsearch for search-heavy endpoints.
- Celery and Redis for background work.

FundingFlow takeaway:

- Keep campaign finance ingestion and API query paths separate. Use prepared views for common public queries.

### LittleSis / Oligrapher

Product precedent:

- Entity relationship database plus visual graph storytelling.
- Graphs need captions, source links, and human-readable annotations.

FundingFlow takeaway:

- The graph should not be a raw hairball. It should support curated paths, timeline filters, source cards, and concise edge labels.

### Splink

Entity-resolution precedent:

- Batch probabilistic linkage.
- Explainable match weights.
- Good fit for large datasets with comparable fields.

FundingFlow takeaway:

- Use Splink for repeatable batch matching after deterministic identifiers are exhausted.

### Dedupe

Entity-resolution precedent:

- Active learning and manual labeling.
- Good fit for training a matching model with uncertain pairs.

FundingFlow takeaway:

- Use Dedupe or a similar review loop for ambiguous entity pairs that need human judgment.

### EDGAR Tools

SEC precedent:

- CIK and ticker lookup helpers.
- Filing retrieval.
- XBRL company facts.
- Local caching and fair-access behavior.

FundingFlow takeaway:

- Reuse patterns for SEC identifiers, caching, and structured facts even if the project implements its own ingestion path.

## Architecture

### Recommended Shape

Use PostgreSQL as the source of truth, then build search and graph read models from it.

Core services:

- Ingestion workers for source-specific import jobs.
- PostgreSQL for normalized records, entity resolution, evidence, and metrics.
- Object storage or local file storage for raw API responses and bulk files.
- OpenSearch or Elasticsearch for full-text and faceted search.
- Optional Neo4j or graph projection tables for multi-hop graph traversal after the relational model is stable.
- Web API for entity pages, search, graph neighborhoods, evidence cards, and metrics.
- Frontend for search, entity profile pages, graph view, timeline view, and source-backed edge inspection.

### Why Not Pure Neo4j First

The hardest early problems are source ingestion, identifier management, uncertain matching, provenance, deduplication, and repeatable metrics. A relational core is better for those constraints. Graph storage can be added as a read model once canonical entities and edges are reliable.

### Chosen Stack

Use Rust for the full backend. Keep PostgreSQL as the source of truth and use React/Next.js for the frontend.

- PostgreSQL with `pg_trgm` for fuzzy candidate search.
- Rust backend API with `axum`, `tokio`, `serde`, `sqlx`, `reqwest`, `tracing`, `thiserror`, and `anyhow`.
- Rust ingestion workers and CLI tools for source imports, normalization, and graph projection jobs.
- SQLx migrations for PostgreSQL schema changes.
- DuckDB for local exploration of large bulk files when it speeds up research, not as the production store.
- Rust-first entity-resolution pipeline for deterministic matching, candidate generation, and review queues.
- Splink or Dedupe only as optional offline research tools if Rust-first matching is not enough.
- Next.js for the frontend.
- OpenSearch or Elasticsearch when search exceeds PostgreSQL full-text needs.

### Lint And Type Gates

Every implementation phase should add or preserve these checks.

Rust backend gates:

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo sqlx prepare --workspace --check` when SQLx offline query checking is enabled.

TypeScript and Next.js gates:

- `pnpm lint` using `eslint .` (Next.js 16 removed `next lint`; use ESLint CLI directly).
- `pnpm typecheck` using `tsc --noEmit`.
- `pnpm build` before release handoff.
- Keep TypeScript `strict` enabled.
- Use `eslint-config-next/core-web-vitals` and `eslint-config-next/typescript` flat config.
- ESLint v9.x is required (v10 breaks `eslint-plugin-react` as of June 2026).

Shared gates:

- Add `scripts/self-test` once code exists and make it run the strongest local Rust and TypeScript checks.
- Keep formatting checks separate from tests so failures are easy to diagnose.
- Treat lint warnings as failures in CI.

## Canonical Data Model

### Core Identity Tables

#### `entity`

Canonical person or organization.

Fields:

- `id`
- `entity_type`: organization, person, committee, agency, unknown
- `display_name`
- `canonical_name`
- `created_at`
- `updated_at`

#### `entity_identifier`

External identifier attached to an entity.

Fields:

- `id`
- `entity_id`
- `source`: usaspending, sam, lda, fec, sec, federal_register, regulations_gov
- `identifier_type`: UEI, CIK, EIN, FEC committee ID, LDA registrant ID, LDA client ID, agency slug, docket ID
- `identifier_value`
- `valid_from`
- `valid_to`
- `evidence_id`

#### `entity_alias`

Name variant attached to an entity.

Fields:

- `id`
- `entity_id`
- `alias`
- `normalized_alias`
- `source`
- `evidence_id`

#### `identity_match`

Candidate or accepted match between two source records or entities.

Fields:

- `id`
- `left_entity_id`
- `right_entity_id`
- `status`: candidate, accepted, rejected, needs_review
- `confidence`
- `method`: deterministic_identifier, pg_trgm, splink, dedupe, manual
- `features_json`
- `reviewed_by`
- `reviewed_at`

### Provenance Tables

#### `source_record`

Raw source object or imported row.

Fields:

- `id`
- `source`
- `source_record_type`
- `source_record_id`
- `source_url`
- `retrieved_at`
- `content_hash`
- `raw_storage_path`

#### `evidence`

Citation unit for an entity field, metric, or edge.

Fields:

- `id`
- `source_record_id`
- `quote_or_field_path`
- `source_url`
- `retrieved_at`
- `confidence`

### Relationship Tables

Use a typed edge table first. Split into specialized tables later only when query or integrity needs require it.

#### `relationship_edge`

Fields:

- `id`
- `from_entity_id`
- `to_entity_id`
- `edge_type`
- `started_on`
- `ended_on`
- `amount`
- `currency`
- `description`
- `confidence`
- `evidence_id`
- `source`

Recommended edge types:

- `recipient_received_federal_obligation`
- `agency_obligated_award_to_recipient`
- `registrant_lobbied_for_client`
- `lobbyist_listed_on_filing`
- `client_reported_lobbying_issue`
- `committee_received_contribution`
- `person_contributed_to_committee`
- `employer_reported_on_contribution`
- `lobbyist_disclosed_contribution_to_committee`
- `company_filed_sec_report`
- `company_reported_revenue_fact`
- `company_disclosed_subsidiary`
- `agency_published_rulemaking_document`
- `entity_submitted_rulemaking_comment`

### Domain Tables

#### Awards

Use for USAspending records.

Fields:

- `award_id`
- `generated_unique_award_id`
- `recipient_entity_id`
- `awarding_agency_entity_id`
- `funding_agency_entity_id`
- `award_type`
- `description`
- `period_start`
- `period_end`
- `obligation_amount`
- `outlay_amount`
- `place_of_performance`
- `evidence_id`

#### Lobbying Filings

Use for LDA records.

Fields:

- `filing_uuid`
- `registrant_entity_id`
- `client_entity_id`
- `filing_type`
- `filing_period`
- `filing_year`
- `income_or_expense`
- `amount`
- `issues_json`
- `evidence_id`

#### Campaign Finance Transactions

Use for FEC records.

Fields:

- `transaction_id`
- `committee_entity_id`
- `contributor_entity_id`
- `candidate_entity_id`
- `amount`
- `date`
- `employer_text`
- `occupation_text`
- `transaction_type`
- `evidence_id`

#### SEC Facts

Use for EDGAR company facts and filing metadata.

Fields:

- `cik`
- `company_entity_id`
- `accession_number`
- `form`
- `filed_at`
- `taxonomy`
- `fact_name`
- `unit`
- `value`
- `period_start`
- `period_end`
- `frame`
- `evidence_id`

#### Rulemaking Records

Use for Federal Register and Regulations.gov records.

Fields:

- `document_id`
- `docket_id`
- `agency_entity_id`
- `title`
- `document_type`
- `publication_date`
- `comment_start_date`
- `comment_end_date`
- `source_url`
- `evidence_id`

## Entity Resolution Plan

### Resolution Order

1. Deterministic identifiers.
2. Exact normalized-name matches within source and jurisdiction constraints.
3. `pg_trgm` fuzzy candidate generation.
4. Splink batch probabilistic linkage for high-volume sources.
5. Dedupe or manual review for uncertain pairs.
6. Human approval for high-impact public graph edges.

### Strong Identifiers

- UEI for SAM and many federal award recipients.
- CIK for SEC registrants.
- FEC committee ID for committees.
- LDA registrant, client, lobbyist, and filing IDs inside LDA.
- Docket IDs for Regulations.gov records.

### Weak or Noisy Identifiers

- Organization names entered as campaign contribution employer text.
- Person names without address or employer context.
- DBA names and subsidiaries.
- Lobbying client names that differ from SEC registrant names.

### Candidate Features

Use these fields when creating match candidates:

- Normalized legal name.
- Alias and DBA names.
- Address components.
- City, state, ZIP.
- Domain or website if available.
- UEI, CIK, EIN, FEC ID, LDA ID when present.
- Parent or subsidiary names.
- Filing source and record date.

### Review UX Requirements

The project should include a review queue early, even if simple.

Each candidate pair should show:

- Both source names.
- Source IDs.
- Address fields.
- Match features and confidence.
- Evidence links.
- Accept, reject, and defer actions.

## Metrics

### MVP Metrics

- Total federal obligations by entity and year.
- Federal obligations by agency, award type, and year.
- Lobbying amount by client, registrant, issue area, and quarter.
- Number of LDA filings by client and year.
- FEC contributions by committee, contributor, employer text, and year.
- SEC reported revenue by company and fiscal period.
- Federal obligations as share of reported revenue.
- Number of rulemaking documents or comments linked to an entity.

### Metric Rules

- Every displayed metric must link to source records.
- Every aggregate must show date range, sources used, and known exclusions.
- Metrics that depend on entity matching must show match confidence.
- Show missing data as missing, not zero.

## Graph Product Requirements

### Entity Page

Each entity page should show:

- Canonical name and aliases.
- Known source identifiers.
- Source-backed relationship summary.
- Timeline of awards, lobbying filings, contributions, SEC filings, and rulemaking records.
- Source cards for each relationship.
- Match-confidence warnings where needed.

### Graph View

The graph view should support:

- One-hop neighborhood around an entity.
- Typed edge filters.
- Date range filter.
- Amount threshold filter.
- Confidence threshold filter.
- Source filter.
- Edge click to open evidence.
- Exportable source list.

### Search

Search should support:

- Organization names.
- Person names.
- UEI.
- CIK.
- FEC committee ID.
- LDA client or registrant names.
- Award IDs.
- Docket IDs.

## MVP Phases

### Phase 0: Project Harness

Deliverables:

- Repo skeleton.
- Database schema migration setup.
- Ingestion job interface.
- Source record and evidence tables.
- Basic tests for normalization and evidence persistence.
- Seed script for a small, documented sample dataset.

Definition of done:

- A clean install can create the database, run migrations, ingest sample records, and run tests.

### Phase 1: USAspending and SAM Core

Deliverables:

- Import USAspending award records.
- Import or map SAM entity records.
- Create canonical entities from recipients and agencies.
- Build entity pages for recipients and agencies.
- Show obligation totals over time.

Definition of done:

- User can search a recipient, open an entity page, inspect awards, and click source evidence.

### Phase 2: LDA Lobbying Layer

Deliverables:

- Import LDA clients, registrants, lobbyists, and filings.
- Link LDA clients to canonical organization entities with confidence scores.
- Show lobbying filings and issue areas on entity pages.
- Add graph edges for lobbying relationships.

Definition of done:

- User can see federal awards and lobbying filings for the same matched organization with evidence and match confidence.

### Phase 3: FEC Campaign Finance Layer

Deliverables:

- Import selected FEC bulk files.
- Model committees, candidates, contributors, employer text, and lobbyist bundled records.
- Link committees and organizations conservatively.
- Add campaign-finance edge types with strong disclaimers.

Definition of done:

- User can filter campaign-finance edges separately from lobbying and spending edges, and can inspect the source record for each edge.

### Phase 4: SEC Disclosures

Deliverables:

- Import SEC submissions and company facts for selected companies.
- Link CIKs to canonical organization entities.
- Display revenue facts with accession and period.
- Compute federal obligations as share of reported revenue.

Definition of done:

- User can compare federal obligations to reported revenue for matched public companies with a clear date range and citation trail.

### Phase 5: Rulemaking Layer

Deliverables:

- Import Federal Register documents for selected agencies and topics.
- Import Regulations.gov docket and comment records after API key setup.
- Link organizations to comments or docket participation where the source supports it.
- Add timeline and graph edges for rulemaking participation.

Definition of done:

- User can view rulemaking events near spending, lobbying, and disclosure events without causal language.

### Phase 6: Graph Export and Portfolio Polish

Deliverables:

- Shareable entity URLs.
- Source-backed graph export.
- Example case studies.
- Data lineage page.
- Methodology page.
- Public README.

Definition of done:

- A reviewer can run the project, inspect the methods, and follow citations from graph edges back to public records.

## Example User Flows

### Trace A Federal Contractor

1. Search for a company name or UEI.
2. Open the entity page.
3. Review federal obligations by year and agency.
4. Add lobbying filings to the timeline.
5. Inspect evidence for each award and filing.
6. Export the source list.

### Compare Public Companies

1. Search for a public company by name or ticker-derived CIK lookup.
2. Open the company page.
3. Review SEC reported revenue facts.
4. Compare federal obligations to reported revenue by fiscal year.
5. Inspect the source records and match confidence.

### Review a Match Candidate

1. Open the entity-resolution review queue.
2. Compare source names, identifiers, addresses, and feature scores.
3. Accept or reject the match.
4. Rebuild affected entity pages and graph edges.

## Risk Register

| Risk | Mitigation |
| --- | --- |
| Entity matching creates false links | Keep match confidence, review queue, evidence links, and conservative thresholds. |
| Users infer causation from proximity | Use neutral labels, disclaimers, timeline wording, and source cards. |
| Bulk datasets are large | Start with bounded slices, use DuckDB for exploration, and add batch jobs before full imports. |
| APIs change or require keys | Prefer bulk files where available, store raw records, and isolate source-specific clients. |
| SEC CORS blocks browser calls | Ingest SEC data in the backend, not from the frontend. |
| FEC employer text is noisy | Treat employer text as weak evidence and avoid automatic organization links without review. |
| Rulemaking comments can include personal data | Store only needed public fields, avoid private enrichment, and document source policy. |
| Metrics mix incompatible periods | Require explicit date windows and source coverage notes for every aggregate. |

## Ethics and Safety Requirements

- Use public records only.
- Cite every edge and aggregate.
- Keep uncertainty visible.
- Keep match review decisions auditable.
- Do not publish speculative allegations.
- Separate facts from interpretation.
- Provide source coverage notes and known gaps.
- Avoid ranking people or organizations by implied suspicion.

## Resume Value

FundingFlow can demonstrate:

- Public-data ingestion from multiple federal sources.
- Data modeling for provenance-heavy systems.
- Entity resolution with deterministic and probabilistic matching.
- Search and graph read models built from a relational core.
- Responsible data presentation with evidence and uncertainty.
- Full-stack product work around complex public records.

## Political-Economy Expansion

FundingFlow should become a political-economy data platform, not only a contracts and lobbying graph.

The expanded product question is:

> Where does public money go, and what happens to the workers and communities around it?

The product has two connected layers:

1. Power flow layer:
   `Company -> federal contract -> agency -> lobbying -> politician/committee -> subsidiary -> executive -> SEC filing`
2. Economic conditions layer:
   `State/county -> wages -> unemployment -> rent -> housing prices -> food prices -> gas -> poverty -> industry mix -> public spending`

The first implementation target is the State MVP: national pulse plus 50-state profiles. Company-to-county and Philly-specific work comes after the state schema, importers, and API are stable.

### State MVP Pages

The homepage should become a national pulse and search entry point:

- Unemployment rate.
- Payroll jobs added or lost.
- CPI and food CPI.
- Food at home and food away from home.
- Gasoline price.
- Selected food price examples with explicit U.S. or regional scope.
- Real hourly earnings.
- Federal minimum wage.
- Search for entities and geographies.

Each state profile should show:

- Labor: unemployment, job growth, average weekly wage, employment by industry.
- Wages: state minimum wage, average earnings, wage growth versus CPI.
- Housing: median home value, median gross rent, FHFA HPI growth, HUD 1BR/2BR fair market rent, rent burden.
- Food and energy: food CPI region, selected average prices, regular gasoline and diesel.
- Economic structure: GDP by industry, personal income, transfer receipts, employment by industry.
- Public money and power: federal obligations into the state, top recipient firms, agencies, lobbying links when matched.

### Data Source Stack

Priority 1 sources:

- BLS API, LAUS, CES, QCEW, CPI, average prices.
- DOL Wage and Hour Division state minimum wage data.
- BEA Regional Economic Accounts.
- Census ACS 5-year.
- FHFA House Price Index.
- HUD Fair Market Rents API.
- EIA gasoline and diesel data.
- USAspending state-level award and obligation data.

Priority 2 sources:

- Census SAIPE.
- Census County Business Patterns.
- IRS SOI county data.
- USDA ERS Food Price Outlook.
- BLS Consumer Expenditure Survey.
- DOL National Database of Childcare Prices.

Priority 3 sources:

- New York Fed Household Debt and Credit.
- FDIC Summary of Deposits.
- FFIEC HMDA.
- NLRB data.
- OSHA and BLS workplace injury data.
- CDC WONDER, CDC PLACES, EPA AQS, EPA TRI.

Implementation constraints:

- Use original agency sources as canonical, even if FRED is later used as a shortcut.
- Store `release_date` and `vintage_date`; do not overwrite historical knowledge with revised data.
- Store evidence for every imported metric and every derived metric input.
- Show missing as missing, not zero.
- Do not present regional or national food prices as state-level prices.
- Treat local minimum wage exceptions, tipped rules, scheduled increases, and notes as structured caveats.
- API-key requirements must produce explicit skipped-import summaries, not silent failures.

### Economic Data Model

Add these tables alongside the existing entity/evidence graph:

```sql
geo (
  geo_id text primary key,
  geo_type text not null,
  name text not null,
  state_code text,
  county_code text,
  region text
);

data_source (
  source_id text primary key,
  agency text not null,
  dataset text not null,
  url text not null,
  license text,
  update_frequency text,
  requires_api_key boolean not null default false,
  notes text
);

metric (
  metric_id text primary key,
  name text not null,
  category text not null,
  unit text not null,
  seasonal_adjustment text,
  source_id text not null references data_source(source_id),
  notes text
);

metric_observation (
  metric_id text not null references metric(metric_id),
  geo_id text not null references geo(geo_id),
  date date not null,
  value numeric not null,
  vintage_date date not null,
  release_date date,
  source_series_id text,
  evidence_id uuid references evidence(id),
  notes text,
  primary key(metric_id, geo_id, date, vintage_date)
);

derived_metric_observation (
  metric_id text not null,
  geo_id text not null references geo(geo_id),
  date date not null,
  value numeric not null,
  formula text not null,
  input_metric_ids text[] not null,
  vintage_date date not null,
  notes text,
  primary key(metric_id, geo_id, date, vintage_date)
);
```

Extend `award` with geography:

```sql
recipient_geo_id text references geo(geo_id);
place_geo_id text references geo(geo_id);
```

For the first slice, state-level mapping is enough. County geocoding and facility matching wait until the county/company phase.

### Derived Indicators

First-class MVP indicators:

```txt
rent_hours_min_wage =
  monthly_2br_fmr / state_min_wage

rent_hours_avg_wage =
  monthly_2br_fmr / avg_hourly_wage

food_pressure =
  food_at_home_cpi_yoy - avg_hourly_earnings_yoy

contract_intensity =
  federal_contract_obligations / state_gdp

public_money_per_worker =
  federal_contract_obligations / total_employment

housing_wage_squeeze =
  rent_growth_yoy - wage_growth_yoy
```

Later indicators:

```txt
reproduction_wage_gap
surplus_population_pressure
labor_share
profit_wage_ratio
productivity_pay_gap
contract_monopoly_ratio
finance_extraction_pressure
tax_burden_low_income
public_debt_interest_burden
contract_vs_welfare_ratio
landlord_concentration
displacement_pressure
exploitation_risk_index
labor_power_score
racialized_labor_gap
carceral_labor_pressure
social_reproduction_crisis_score
```

### Rust Backend Implementation

The Rust backend is in scope and should remain the core system:

- Add domain types for geographies, data sources, metrics, observations, derived metrics, and state profile responses.
- Add SQLx migrations for economic tables and award geography columns.
- Add query functions in `db` for:
  - upserting geographies, data sources, metrics, and observations;
  - listing state profiles;
  - fetching latest observations by category;
  - aggregating public-money totals by state;
  - writing derived metrics.
- Add importers in `ingestion`:
  - `bls_laus`
  - `bls_ces`
  - `bls_cpi_prices`
  - `dol_min_wage`
  - `bea_regional`
  - `census_acs`
  - `fhfa_hpi`
  - `hud_fmr`
  - `eia_gas`
  - `usaspending_state`
- Add CLI commands for economic imports:
  - `cargo run --bin cli -- import --source bls-laus`
  - `cargo run --bin cli -- import --source dol-min-wage`
  - `cargo run --bin cli -- import --source bea-regional`
  - `cargo run --bin cli -- import --source census-acs`
  - `cargo run --bin cli -- import --source fhfa-hpi`
  - `cargo run --bin cli -- import --source hud-fmr`
  - `cargo run --bin cli -- import --source eia-gas`
  - `cargo run --bin cli -- derive --metric state-mvp`
- Add API endpoints:
  - `GET /api/v1/pulse/national`
  - `GET /api/v1/geos/search?q=...`
  - `GET /api/v1/geos/{geo_id}/profile`
  - `GET /api/v1/geos/{geo_id}/metrics?category=...`
  - `GET /api/v1/geos/{geo_id}/public-money`

API implementation should keep using axum `State` with shared `PgPool`, typed query functions in `db`, and explicit HTTP error mapping. SQLx migrations must be covered by tests or migration-backed integration checks.

### Frontend Implementation

Keep Next.js and TypeScript strict mode.

Add:

- National pulse section on `/`.
- Geography search result type alongside entity results.
- State profile page at `/states/[stateCode]`.
- Source and vintage labels on every metric card.
- Missing-data states for unavailable metrics.
- Clear scope labels for food prices: U.S. average, regional average, metro when available.
- Data Lineage updates for the economic-condition pipeline.
- Methodology updates explaining vintage data, derived metrics, and limitations.

The interface should feel dense and operational, not like a school project. It should avoid implying causal guilt and should keep source-backed facts separate from interpretation.

### Strict Quality Kit

Use the strict repo-rule-kit approach.

Implementation tasks:

- Fix current `cargo fmt --check` drift.
- Keep `scripts/self-test` as the canonical gate.
- Add `verify.sh` as a thin repo-level wrapper.
- Add Cargo workspace lint policy.
- Add `sgconfig.yml` for structural linting.
- Add `.codex/hooks.json` and `.codex/hooks/stop_quality_gate.py`.
- Add `.opencode/commands/check.md` and `.opencode/commands/verify.md`.
- Add `docs/agent/rules.md`.
- Update `scripts/self-test` to run Rust and frontend checks:
  - `cargo fmt --all --check`
  - `cargo check --workspace --all-targets --all-features`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-features`
  - `pnpm --dir apps/web lint`
  - `pnpm --dir apps/web typecheck`
  - `pnpm --dir apps/web build`
- Make frontend builds deterministic. Either replace remote `next/font/google` fetching with local/system font handling or commit local font assets.

### Verification Plan

Required checks before completion:

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `pnpm --dir apps/web lint`
- `pnpm --dir apps/web typecheck`
- `pnpm --dir apps/web build`
- `./scripts/self-test`

Test coverage:

- Unit tests for metric IDs, geography IDs, derived formulas, and importer parsing helpers.
- Fixture-based importer tests for BLS, DOL, BEA, Census, HUD, FHFA, EIA, and USAspending response shapes.
- Query tests for latest-observation selection, missing values, state public-money totals, and derived metric calculation.
- Frontend checks for national pulse, state profile rendering, empty states, and correct source/scope labels.

### Immediate Build Plan

1. Initialize the repository as a git repo.
2. Add this political-economy context to `plan.md`.
3. Fix verification drift and wire the strict quality kit.
4. Add economic schema migrations and domain types.
5. Add Rust query functions and seed/fixture data for state profiles.
6. Add MVP economic importers with fixture tests and graceful missing-key behavior.
7. Add derived metrics for rent, food, public money, and housing squeeze.
8. Add API endpoints for national pulse, geography search, state profile, metric lists, and public money.
9. Add frontend national pulse and state profile pages.
10. Run the full verification plan and document any remaining external-data limitations.

### Implementation Ledger

Current implemented slice:

- Repository initialized with Git. No commit has been made yet.
- Strict quality kit added: Rust formatting/check/clippy/tests, frontend lint/typecheck/build, `scripts/self-test`, `verify.sh`, agent rules, and stop-hook quality gate.
- Rust backend added as the core backend piece: `axum` API, `sqlx` Postgres queries, Rust ingestion crate, and CLI import/derive commands.
- Economic schema added for geographies, sources, metrics, observations, derived metrics, and award geography links.
- State MVP API added: national pulse, geography search, state profile, metric lists, and public-money summary endpoints.
- Next.js state profile UI added with state-level economic metrics, derived pressure metrics, public-money summary, and source notes.
- Live no-key importers implemented and verified locally:
  - BLS LAUS state unemployment.
  - BLS CES national total nonfarm payroll change and total private average hourly earnings.
  - BLS CPI food-at-home and food-away-from-home YoY plus U.S. city average basket prices for eggs, milk, white bread, ground beef, chicken breast, and bananas.
  - DOL state minimum wage and effective minimum wage, with federal-floor handling for states with no or lower state basic rate.
  - FHFA quarterly all-transactions state HPI YoY, not seasonally adjusted.
  - USAspending state federal award obligations.
- Keyed importer implemented with safe no-key skip:
  - Census ACS 2024 5-year state income, rent, rent burden, and poverty metrics.
- Fixture importer remains for missing-key/local development values and national pulse placeholders.
- Derived state MVP metrics implemented: rent hours at effective minimum wage, rent hours at average hourly earnings, and contract intensity.
- Derived national cost-basket metrics implemented: BLS food basket cost and food basket as a percent of average weekly wage.
- Verified with local Postgres imports, direct SQL inspection, API endpoint checks, frontend visual checks, and `./scripts/self-test`.

Known implementation gaps still in scope:

- Replace fixture placeholders with live or keyed imports for BLS CES/CPI average prices, BEA Regional Accounts, Census ACS, HUD FMR, EIA gas, USDA ERS, SAIPE, IRS SOI, union/work stoppage, injury, health, debt, finance, public-finance, concentration, and local property/eviction layers.
- Add company/facility/county joins from award recipients and places of performance into the economic-conditions layer.
- Add county pages, cost basket page, company economic-conditions panel, and national pulse live CPI/price data.
- Add stronger database integration tests that run against a disposable Postgres instance instead of staying ignored by default.

## Reference Links

- USAspending API docs: `https://api.usaspending.gov/docs/`
- USAspending source: `https://github.com/fedspendingtransparency/usaspending-api`
- LDA API: `https://lda.gov/api/v1/`
- SAM Entity API: `https://open.gsa.gov/api/entity-api/`
- SAM Entity Extracts API: `https://open.gsa.gov/api/sam-entity-extracts-api/`
- GSA Contract Awards API: `https://open.gsa.gov/api/contract-awards/`
- GSA Opportunities API: `https://open.gsa.gov/api/get-opportunities-public-api/`
- GSA Federal Hierarchy API: `https://open.gsa.gov/api/fh-public-api/`
- OpenFEC source: `https://github.com/fecgov/openFEC`
- SEC data APIs: `https://data.sec.gov/`
- Federal Register API: `https://www.federalregister.gov/api/v1/documents.json`
- Regulations.gov API docs: `https://open.gsa.gov/api/regulationsgov/`
- LobbyView: `https://www.lobbyview.org/`
- LobbyView docs: `https://lobbyview.readthedocs.io/en/latest/api.html`
- LittleSis Rails: `https://github.com/public-accountability/littlesis-rails`
- Oligrapher: `https://github.com/public-accountability/oligrapher`
- Splink: `https://github.com/moj-analytical-services/splink`
- Dedupe: `https://github.com/dedupeio/dedupe`
- EDGAR Tools: `https://github.com/dgunning/edgartools`
- BLS API docs: `https://www.bls.gov/developers/`
- BLS QCEW: `https://www.bls.gov/cew/`
- DOL state minimum wages: `https://www.dol.gov/agencies/whd/minimum-wage/state`
- BEA API docs: `https://apps.bea.gov/API/docs/`
- Census ACS API: `https://www.census.gov/data/developers/data-sets/acs-5year.html`
- Census API data sets: `https://www.census.gov/data/developers/data-sets.html`
- FHFA HPI: `https://www.fhfa.gov/data/hpi`
- HUD FMR API: `https://www.huduser.gov/portal/dataset/fmr-api.html`
- EIA API docs: `https://www.eia.gov/opendata/documentation.php`
- EIA gasoline and diesel update: `https://www.eia.gov/petroleum/gasdiesel/`
- USDA ERS Food Price Outlook: `https://www.ers.usda.gov/data-products/food-price-outlook`
- IRS SOI county data: `https://www.irs.gov/statistics/soi-tax-stats-county-data`
- Census SAIPE: `https://www.census.gov/programs-surveys/saipe.html`
