export interface Entity {
  id: string;
  entity_type: string;
  display_name: string;
  canonical_name: string;
  created_at: string;
  updated_at: string;
}

export interface EntityIdentifier {
  id: string;
  entity_id: string;
  source: string;
  identifier_type: string;
  identifier_value: string;
  valid_from: string | null;
  valid_to: string | null;
  evidence_id: string | null;
}

export interface EntityAlias {
  id: string;
  entity_id: string;
  alias: string;
  normalized_alias: string;
  source: string;
  evidence_id: string | null;
}

export interface EntityWithDetails {
  entity: Entity;
  identifiers: EntityIdentifier[];
  aliases: EntityAlias[];
}

export interface EntitySummary {
  id: string;
  entity_type: string;
  display_name: string;
  identifier_count: number;
  award_count: number;
  total_obligations: number | null;
}

export interface SearchResult {
  entities: EntitySummary[];
  total: number;
}

export interface Award {
  id: string;
  generated_unique_award_id: string;
  recipient_entity_id: string | null;
  awarding_agency_entity_id: string | null;
  funding_agency_entity_id: string | null;
  award_type: string | null;
  description: string | null;
  period_start: string | null;
  period_end: string | null;
  obligation_amount: number | null;
  outlay_amount: number | null;
  place_of_performance: string | null;
  recipient_geo_id: string | null;
  place_geo_id: string | null;
  evidence_id: string | null;
}

export interface LobbyingFiling {
  id: string;
  filing_uuid: string;
  registrant_entity_id: string | null;
  client_entity_id: string | null;
  filing_type: string | null;
  filing_period: string | null;
  filing_year: number | null;
  income_or_expense: string | null;
  amount: number | null;
  issues_json: unknown;
  evidence_id: string | null;
}

export interface RelationshipEdge {
  id: string;
  from_entity_id: string;
  to_entity_id: string;
  edge_type: string;
  started_on: string | null;
  ended_on: string | null;
  amount: number | null;
  currency: string | null;
  description: string | null;
  confidence: number | null;
  evidence_id: string | null;
  source: string;
}

export interface Geo {
  geo_id: string;
  geo_type: string;
  name: string;
  state_code: string | null;
  county_code: string | null;
  region: string | null;
}

export interface DataSource {
  source_id: string;
  agency: string;
  dataset: string;
  url: string;
  license: string | null;
  update_frequency: string | null;
  requires_api_key: boolean;
  notes: string | null;
}

export interface Metric {
  metric_id: string;
  name: string;
  category: string;
  unit: string;
  seasonal_adjustment: string | null;
  source_id: string;
  notes: string | null;
}

export interface MetricObservation {
  metric_id: string;
  geo_id: string;
  date: string;
  value: number;
  vintage_date: string;
  release_date: string | null;
  source_series_id: string | null;
  evidence_id: string | null;
  notes: string | null;
}

export interface MetricObservationWithDetails {
  observation: MetricObservation;
  metric: Metric;
  source: DataSource;
}

export interface DerivedMetricObservation {
  metric_id: string;
  geo_id: string;
  date: string;
  value: number;
  formula: string;
  input_metric_ids: string[];
  vintage_date: string;
  notes: string | null;
}

export interface PublicMoneyRecipient {
  entity_id: string | null;
  display_name: string;
  total_obligations: number;
  award_count: number;
}

export interface PublicMoneySummary {
  geo_id: string;
  total_obligations: number;
  award_count: number;
  top_recipients: PublicMoneyRecipient[];
}

export interface StateProfile {
  geo: Geo;
  latest_metrics: MetricObservationWithDetails[];
  derived_metrics: DerivedMetricObservation[];
  public_money: PublicMoneySummary;
}

export interface NationalPulse {
  geo: Geo;
  latest_metrics: MetricObservationWithDetails[];
  derived_metrics: DerivedMetricObservation[];
}

export interface GeoSearchResult {
  geos: Geo[];
  total: number;
}

export interface UnifiedSearchResult {
  entities: EntitySummary[];
  entity_total: number;
  geos: Geo[];
  geo_total: number;
}

const API_BASE =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

export async function unifiedSearch(
  q: string,
  limit = 20,
  offset = 0
): Promise<UnifiedSearchResult> {
  const params = new URLSearchParams({ q, limit: String(limit), offset: String(offset) });
  const res = await fetch(`${API_BASE}/api/v1/search?${params}`);
  if (!res.ok) throw new Error(`unified search failed: ${res.status}`);
  return res.json();
}

export async function searchEntities(
  q: string,
  limit = 20,
  offset = 0
): Promise<SearchResult> {
  const params = new URLSearchParams({ q, limit: String(limit), offset: String(offset) });
  const res = await fetch(`${API_BASE}/api/v1/entities/search?${params}`);
  if (!res.ok) throw new Error(`search failed: ${res.status}`);
  return res.json();
}

export async function getEntity(id: string): Promise<EntityWithDetails> {
  const res = await fetch(`${API_BASE}/api/v1/entities/${id}`);
  if (!res.ok) throw new Error(`entity fetch failed: ${res.status}`);
  return res.json();
}

export async function getEntityAwards(entityId: string): Promise<Award[]> {
  const res = await fetch(`${API_BASE}/api/v1/entities/${entityId}/awards`);
  if (!res.ok) throw new Error(`awards fetch failed: ${res.status}`);
  return res.json();
}

export async function getEntityLobbying(entityId: string): Promise<LobbyingFiling[]> {
  const res = await fetch(`${API_BASE}/api/v1/entities/${entityId}/lobbying`);
  if (!res.ok) throw new Error(`lobbying fetch failed: ${res.status}`);
  return res.json();
}

export async function getEntityEdges(entityId: string): Promise<RelationshipEdge[]> {
  const res = await fetch(`${API_BASE}/api/v1/entities/${entityId}/edges`);
  if (!res.ok) throw new Error(`edges fetch failed: ${res.status}`);
  return res.json();
}

export async function getEntityEconomicContext(
  entityId: string
): Promise<StateProfile[]> {
  const res = await fetch(`${API_BASE}/api/v1/entities/${entityId}/economic-context`);
  if (!res.ok) throw new Error(`economic context fetch failed: ${res.status}`);
  return res.json();
}

export async function getNationalPulse(): Promise<NationalPulse> {
  const res = await fetch(`${API_BASE}/api/v1/pulse/national`, {
    next: { revalidate: 60 },
  });
  if (!res.ok) throw new Error(`national pulse fetch failed: ${res.status}`);
  return res.json();
}

export async function searchGeos(q: string, limit = 20): Promise<GeoSearchResult> {
  const params = new URLSearchParams({ q, limit: String(limit) });
  const res = await fetch(`${API_BASE}/api/v1/geos/search?${params}`);
  if (!res.ok) throw new Error(`geo search failed: ${res.status}`);
  return res.json();
}

export async function getStateProfile(geoId: string): Promise<StateProfile> {
  const res = await fetch(`${API_BASE}/api/v1/geos/${geoId}/profile`, {
    next: { revalidate: 60 },
  });
  if (!res.ok) throw new Error(`state profile fetch failed: ${res.status}`);
  return res.json();
}

export async function getMetricHistory(
  geoId: string,
  metricId: string,
  from?: string,
  to?: string,
  limit = 500
): Promise<MetricObservationWithDetails[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  if (from) params.set("from", from);
  if (to) params.set("to", to);
  const res = await fetch(
    `${API_BASE}/api/v1/geos/${geoId}/metrics/${metricId}/history?${params}`,
    { next: { revalidate: 60 } }
  );
  if (!res.ok) throw new Error(`metric history fetch failed: ${res.status}`);
  return res.json();
}

export function formatMoney(amount: number | null): string {
  if (amount == null) return "N/A";
  if (amount >= 1_000_000_000) {
    return `$${(amount / 1_000_000_000).toFixed(2)}B`;
  }
  if (amount >= 1_000_000) {
    return `$${(amount / 1_000_000).toFixed(1)}M`;
  }
  if (amount < 1_000) {
    return `$${amount.toFixed(2)}`;
  }
  return `$${amount.toLocaleString()}`;
}

export function formatMetricValue(value: number, unit: string): string {
  if (unit === "percent") return `${value.toFixed(1)}%`;
  if (unit === "usd") return formatMoney(value);
  if (unit === "usd_per_hour") return `$${value.toFixed(2)}/hr`;
  if (unit === "usd_per_month") return `$${Math.round(value).toLocaleString()}/mo`;
  if (unit === "usd_per_gallon") return `$${value.toFixed(2)}/gal`;
  if (unit === "usd_per_dozen") return `$${value.toFixed(2)}/doz`;
  if (unit === "usd_per_lb") return `$${value.toFixed(2)}/lb`;
  if (unit === "usd_per_week") return `$${Math.round(value).toLocaleString()}/wk`;
  if (unit === "jobs") return value.toLocaleString();
  return value.toLocaleString();
}

export function derivedMetricLabel(metricId: string): string {
  const labels: Record<string, string> = {
    rent_hours_min_wage: "Hours at minimum wage for 2BR rent",
    rent_hours_avg_wage: "Hours at average wage for 2BR rent",
    contract_intensity: "Federal obligations as share of GDP",
    food_basket_cost: "Food basket cost",
    food_basket_as_percent_weekly_wage: "Food basket share of weekly wage",
  };
  return labels[metricId] ?? metricId;
}

export function edgeTypeLabel(edgeType: string): string {
  const labels: Record<string, string> = {
    recipient_received_federal_obligation: "Received federal obligation",
    agency_obligated_award_to_recipient: "Agency obligated award",
    registrant_lobbied_for_client: "Registrant lobbied for client",
    lobbyist_listed_on_filing: "Lobbyist listed on filing",
    client_reported_lobbying_issue: "Client reported lobbying issue",
    committee_received_contribution: "Committee received contribution",
    person_contributed_to_committee: "Person contributed to committee",
    employer_reported_on_contribution: "Employer reported on contribution",
    lobbyist_disclosed_contribution_to_committee: "Lobbyist disclosed contribution",
    company_filed_sec_report: "Company filed SEC report",
    company_reported_revenue_fact: "Company reported revenue",
    company_disclosed_subsidiary: "Company disclosed subsidiary",
    agency_published_rulemaking_document: "Agency published document",
    entity_submitted_rulemaking_comment: "Submitted comment",
    committee_contributed_to_candidate: "Committee contributed to candidate",
    committee_transferred_to_committee: "Committee transferred to committee",
    candidate_supported_by_committee: "Candidate supported by committee",
  };
  return labels[edgeType] ?? edgeType;
}

// Campaign finance types

export interface CampaignFinanceTransaction {
  id: string;
  transaction_id: string;
  committee_entity_id: string | null;
  contributor_entity_id: string | null;
  candidate_entity_id: string | null;
  amount: number | null;
  date: string | null;
  employer_text: string | null;
  occupation_text: string | null;
  transaction_type: string | null;
  evidence_id: string | null;
  contributor_city: string | null;
  contributor_state: string | null;
  contributor_zip: string | null;
  committee_fec_id: string | null;
  candidate_fec_id: string | null;
  memo_text: string | null;
  transaction_type_code: string | null;
  cycle: number | null;
  recipient_committee_name: string | null;
  other_entity_id: string | null;
  sub_id: string | null;
}

export interface ContributionRecipientSummary {
  recipient_entity_id: string | null;
  recipient_name: string;
  total_amount: number;
  transaction_count: number;
}

export interface ContributionCycleSummary {
  cycle: number;
  total_amount: number;
  transaction_count: number;
}

export interface ContributionSummary {
  total_amount: number;
  transaction_count: number;
  by_recipient: ContributionRecipientSummary[];
  by_cycle: ContributionCycleSummary[];
}

export interface ContributionSearchResult {
  contributor_entity_id: string | null;
  contributor_name: string;
  total_amount: number;
  transaction_count: number;
  top_recipients: ContributionRecipientSummary[];
}

export interface CandidateSearchResult {
  entity_id: string;
  display_name: string;
  candidate_fec_id: string;
  party: string | null;
  office: string | null;
  office_state: string | null;
  election_year: number | null;
}

export interface CandidateInfo {
  entity_id: string;
  candidate_fec_id: string;
  party: string | null;
  office: string | null;
  office_state: string | null;
  office_district: string | null;
  incumbent_challenge: string | null;
  election_year: number | null;
  candidate_status: string | null;
  principal_committee_fec_id: string | null;
}

export interface CommitteeInfo {
  entity_id: string;
  committee_fec_id: string;
  committee_type: string | null;
  committee_designation: string | null;
  party: string | null;
  treasurer_name: string | null;
  organization_type: string | null;
  connected_organization: string | null;
}

export interface MoneyFlowStep {
  committee_entity_id: string | null;
  committee_name: string;
  committee_fec_id: string;
  amount: number;
  transaction_count: number;
}

export interface MoneyFlowResult {
  from_entity_id: string;
  to_candidate_id: string;
  total_amount: number;
  flows: MoneyFlowStep[];
}

export interface EntityContributionsResponse {
  summary: ContributionSummary | null;
  transactions: CampaignFinanceTransaction[];
  candidate_info: CandidateInfo | null;
  committee_info: CommitteeInfo | null;
}

// Campaign finance API functions

export async function searchContributions(
  q: string,
  cycle?: number,
  limit = 20
): Promise<ContributionSearchResult[]> {
  const params = new URLSearchParams({ q, limit: String(limit) });
  if (cycle) params.set("cycle", String(cycle));
  const res = await fetch(`${API_BASE}/api/v1/contributions/search?${params}`);
  if (!res.ok) throw new Error(`contributions search failed: ${res.status}`);
  return res.json();
}

export async function getEntityContributions(
  entityId: string,
  cycle?: number
): Promise<EntityContributionsResponse> {
  const params = new URLSearchParams();
  if (cycle) params.set("cycle", String(cycle));
  const res = await fetch(`${API_BASE}/api/v1/entities/${entityId}/contributions?${params}`);
  if (!res.ok) throw new Error(`contributions fetch failed: ${res.status}`);
  return res.json();
}

export async function searchCandidates(
  q: string,
  limit = 20
): Promise<CandidateSearchResult[]> {
  const params = new URLSearchParams({ q, limit: String(limit) });
  const res = await fetch(`${API_BASE}/api/v1/candidates/search?${params}`);
  if (!res.ok) throw new Error(`candidates search failed: ${res.status}`);
  return res.json();
}

export async function getMoneyFlow(
  fromEntityId: string,
  toCandidateId: string
): Promise<MoneyFlowResult> {
  const params = new URLSearchParams({
    from_entity_id: fromEntityId,
    to_candidate_id: toCandidateId,
  });
  const res = await fetch(`${API_BASE}/api/v1/contributions/flow?${params}`);
  if (!res.ok) throw new Error(`money flow fetch failed: ${res.status}`);
  return res.json();
}

export async function getTopDonors(
  committeeId: string,
  limit = 20
): Promise<ContributionRecipientSummary[]> {
  const params = new URLSearchParams({ committee_id: committeeId, limit: String(limit) });
  const res = await fetch(`${API_BASE}/api/v1/contributions/top-donors?${params}`);
  if (!res.ok) throw new Error(`top donors fetch failed: ${res.status}`);
  return res.json();
}
