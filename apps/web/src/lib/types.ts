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

const API_BASE =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

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
  };
  return labels[edgeType] ?? edgeType;
}
