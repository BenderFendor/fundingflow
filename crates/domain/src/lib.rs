use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "entity_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Organization,
    Person,
    Committee,
    Agency,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub display_name: String,
    pub canonical_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "identifier_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IdentifierSource {
    Usaspending,
    Sam,
    Lda,
    Fec,
    Sec,
    FederalRegister,
    RegulationsGov,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "identifier_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IdentifierType {
    Uei,
    Cik,
    Ein,
    FecCommitteeId,
    LdaRegistrantId,
    LdaClientId,
    LdaLobbyistId,
    AgencySlug,
    DocketId,
    Ticker,
    AwardId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityIdentifier {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub source: IdentifierSource,
    pub identifier_type: IdentifierType,
    pub identifier_value: String,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityAlias {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub alias: String,
    pub normalized_alias: String,
    pub source: IdentifierSource,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "match_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Candidate,
    Accepted,
    Rejected,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "match_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MatchMethod {
    DeterministicIdentifier,
    PgTrgm,
    Splink,
    Dedupe,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityMatch {
    pub id: Uuid,
    pub left_entity_id: Uuid,
    pub right_entity_id: Uuid,
    pub status: MatchStatus,
    pub confidence: Option<f64>,
    pub method: MatchMethod,
    pub features_json: Option<serde_json::Value>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: Uuid,
    pub source: String,
    pub source_record_type: String,
    pub source_record_id: String,
    pub source_url: Option<String>,
    pub retrieved_at: DateTime<Utc>,
    pub content_hash: String,
    pub raw_storage_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub source_record_id: Uuid,
    pub quote_or_field_path: Option<String>,
    pub source_url: Option<String>,
    pub retrieved_at: DateTime<Utc>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "edge_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    RecipientReceivedFederalObligation,
    AgencyObligatedAwardToRecipient,
    RegistrantLobbiedForClient,
    LobbyistListedOnFiling,
    ClientReportedLobbyingIssue,
    CommitteeReceivedContribution,
    PersonContributedToCommittee,
    EmployerReportedOnContribution,
    LobbyistDisclosedContributionToCommittee,
    CompanyFiledSecReport,
    CompanyReportedRevenueFact,
    CompanyDisclosedSubsidiary,
    AgencyPublishedRulemakingDocument,
    EntitySubmittedRulemakingComment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipEdge {
    pub id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub edge_type: EdgeType,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f64>,
    pub evidence_id: Option<Uuid>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Award {
    pub id: Uuid,
    pub generated_unique_award_id: String,
    pub recipient_entity_id: Option<Uuid>,
    pub awarding_agency_entity_id: Option<Uuid>,
    pub funding_agency_entity_id: Option<Uuid>,
    pub award_type: Option<String>,
    pub description: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub obligation_amount: Option<f64>,
    pub outlay_amount: Option<f64>,
    pub place_of_performance: Option<String>,
    pub recipient_geo_id: Option<String>,
    pub place_geo_id: Option<String>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyingFiling {
    pub id: Uuid,
    pub filing_uuid: String,
    pub registrant_entity_id: Option<Uuid>,
    pub client_entity_id: Option<Uuid>,
    pub filing_type: Option<String>,
    pub filing_period: Option<String>,
    pub filing_year: Option<i32>,
    pub income_or_expense: Option<String>,
    pub amount: Option<f64>,
    pub issues_json: Option<serde_json::Value>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignFinanceTransaction {
    pub id: Uuid,
    pub transaction_id: String,
    pub committee_entity_id: Option<Uuid>,
    pub contributor_entity_id: Option<Uuid>,
    pub candidate_entity_id: Option<Uuid>,
    pub amount: Option<f64>,
    pub date: Option<NaiveDate>,
    pub employer_text: Option<String>,
    pub occupation_text: Option<String>,
    pub transaction_type: Option<String>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFact {
    pub id: Uuid,
    pub cik: String,
    pub company_entity_id: Option<Uuid>,
    pub accession_number: String,
    pub form: Option<String>,
    pub filed_at: Option<NaiveDate>,
    pub taxonomy: Option<String>,
    pub fact_name: Option<String>,
    pub unit: Option<String>,
    pub value: Option<f64>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub frame: Option<String>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulemakingRecord {
    pub id: Uuid,
    pub document_id: Option<String>,
    pub docket_id: Option<String>,
    pub agency_entity_id: Option<Uuid>,
    pub title: Option<String>,
    pub document_type: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub comment_start_date: Option<NaiveDate>,
    pub comment_end_date: Option<NaiveDate>,
    pub source_url: Option<String>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityWithDetails {
    pub entity: Entity,
    pub identifiers: Vec<EntityIdentifier>,
    pub aliases: Vec<EntityAlias>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub display_name: String,
    pub identifier_count: i64,
    pub award_count: i64,
    pub total_obligations: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entities: Vec<EntitySummary>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geo {
    pub geo_id: String,
    pub geo_type: String,
    pub name: String,
    pub state_code: Option<String>,
    pub county_code: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub source_id: String,
    pub agency: String,
    pub dataset: String,
    pub url: String,
    pub license: Option<String>,
    pub update_frequency: Option<String>,
    pub requires_api_key: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub metric_id: String,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub seasonal_adjustment: Option<String>,
    pub source_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricObservation {
    pub metric_id: String,
    pub geo_id: String,
    pub date: NaiveDate,
    pub value: f64,
    pub vintage_date: NaiveDate,
    pub release_date: Option<NaiveDate>,
    pub source_series_id: Option<String>,
    pub evidence_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedMetricObservation {
    pub metric_id: String,
    pub geo_id: String,
    pub date: NaiveDate,
    pub value: f64,
    pub formula: String,
    pub input_metric_ids: Vec<String>,
    pub vintage_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoSearchResult {
    pub geos: Vec<Geo>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMoneySummary {
    pub geo_id: String,
    pub total_obligations: f64,
    pub award_count: i64,
    pub top_recipients: Vec<PublicMoneyRecipient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMoneyRecipient {
    pub entity_id: Option<Uuid>,
    pub display_name: String,
    pub total_obligations: f64,
    pub award_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProfile {
    pub geo: Geo,
    pub latest_metrics: Vec<MetricObservationWithDetails>,
    pub derived_metrics: Vec<DerivedMetricObservation>,
    pub public_money: PublicMoneySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalPulse {
    pub geo: Geo,
    pub latest_metrics: Vec<MetricObservationWithDetails>,
    pub derived_metrics: Vec<DerivedMetricObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricObservationWithDetails {
    pub observation: MetricObservation,
    pub metric: Metric,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityRequest {
    pub entity_type: EntityType,
    pub display_name: String,
    pub canonical_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwardResponse {
    pub award: Award,
    pub recipient_name: Option<String>,
    pub awarding_agency_name: Option<String>,
    pub funding_agency_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateEdgeRequest {
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub edge_type: EdgeType,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub confidence: Option<f64>,
    pub evidence_id: Option<Uuid>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct CreateAwardRequest {
    pub generated_unique_award_id: String,
    pub recipient_entity_id: Option<Uuid>,
    pub awarding_agency_entity_id: Option<Uuid>,
    pub funding_agency_entity_id: Option<Uuid>,
    pub award_type: Option<String>,
    pub description: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub obligation_amount: Option<f64>,
    pub outlay_amount: Option<f64>,
    pub place_of_performance: Option<String>,
    pub recipient_geo_id: Option<String>,
    pub place_geo_id: Option<String>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CreateLobbyingFilingRequest {
    pub filing_uuid: String,
    pub registrant_entity_id: Option<Uuid>,
    pub client_entity_id: Option<Uuid>,
    pub filing_type: Option<String>,
    pub filing_period: Option<String>,
    pub filing_year: Option<i32>,
    pub income_or_expense: Option<String>,
    pub amount: Option<f64>,
    pub issues_json: Option<serde_json::Value>,
    pub evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeResponse {
    pub edge: RelationshipEdge,
    pub from_name: String,
    pub to_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_serialization() {
        let variants = [
            ("organization", EntityType::Organization),
            ("person", EntityType::Person),
            ("committee", EntityType::Committee),
            ("agency", EntityType::Agency),
            ("unknown", EntityType::Unknown),
        ];
        for (expected, variant) in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert!(json.contains(expected), "failed for {expected}");
            let back: EntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant);
        }
    }

    #[test]
    fn test_edge_type_serialization() {
        let edges = [
            EdgeType::RecipientReceivedFederalObligation,
            EdgeType::AgencyObligatedAwardToRecipient,
            EdgeType::RegistrantLobbiedForClient,
            EdgeType::LobbyistListedOnFiling,
            EdgeType::ClientReportedLobbyingIssue,
            EdgeType::CommitteeReceivedContribution,
            EdgeType::PersonContributedToCommittee,
            EdgeType::EmployerReportedOnContribution,
            EdgeType::LobbyistDisclosedContributionToCommittee,
            EdgeType::CompanyFiledSecReport,
            EdgeType::CompanyReportedRevenueFact,
            EdgeType::CompanyDisclosedSubsidiary,
            EdgeType::AgencyPublishedRulemakingDocument,
            EdgeType::EntitySubmittedRulemakingComment,
        ];
        for edge in &edges {
            let json = serde_json::to_string(edge).unwrap();
            let back: EdgeType = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, edge, "roundtrip failed for {:?}", edge);
        }
    }

    #[test]
    fn test_identifier_source_serialization() {
        let sources = [
            IdentifierSource::Usaspending,
            IdentifierSource::Sam,
            IdentifierSource::Lda,
            IdentifierSource::Fec,
            IdentifierSource::Sec,
            IdentifierSource::FederalRegister,
            IdentifierSource::RegulationsGov,
        ];
        for source in &sources {
            let json = serde_json::to_string(source).unwrap();
            let back: IdentifierSource = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, source);
        }
    }

    #[test]
    fn test_match_status_serialization() {
        let statuses = [
            MatchStatus::Candidate,
            MatchStatus::Accepted,
            MatchStatus::Rejected,
            MatchStatus::NeedsReview,
        ];
        for s in &statuses {
            let json = serde_json::to_string(s).unwrap();
            let back: MatchStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, s);
        }
    }

    #[test]
    fn test_match_method_serialization() {
        let methods = [
            MatchMethod::DeterministicIdentifier,
            MatchMethod::PgTrgm,
            MatchMethod::Splink,
            MatchMethod::Dedupe,
            MatchMethod::Manual,
        ];
        for m in &methods {
            let json = serde_json::to_string(m).unwrap();
            let back: MatchMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, m);
        }
    }

    #[test]
    fn test_deserialize_entity_summary() {
        let json = r#"{"id":"00000000-0000-0000-0000-000000000001","entity_type":"organization","display_name":"Test Corp","identifier_count":5,"award_count":3,"total_obligations":1.5e6}"#;
        let summary: EntitySummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.display_name, "Test Corp");
        assert_eq!(summary.identifier_count, 5);
        assert_eq!(summary.award_count, 3);
        assert_eq!(summary.total_obligations, Some(1_500_000.0));
    }

    #[test]
    fn test_create_edge_request_fields() {
        let req = CreateEdgeRequest {
            from_entity_id: uuid::Uuid::nil(),
            to_entity_id: uuid::Uuid::nil(),
            edge_type: EdgeType::RecipientReceivedFederalObligation,
            started_on: None,
            ended_on: None,
            amount: Some(1000.0),
            currency: Some("USD".into()),
            description: Some("test".into()),
            confidence: Some(0.9),
            evidence_id: None,
            source: "test".into(),
        };
        assert_eq!(req.source, "test");
        assert_eq!(req.amount, Some(1000.0));
    }

    #[test]
    fn test_create_award_request_defaults() {
        let req = CreateAwardRequest {
            generated_unique_award_id: "AW-001".into(),
            recipient_entity_id: None,
            awarding_agency_entity_id: None,
            funding_agency_entity_id: None,
            award_type: None,
            description: None,
            period_start: None,
            period_end: None,
            obligation_amount: None,
            outlay_amount: None,
            place_of_performance: None,
            recipient_geo_id: None,
            place_geo_id: None,
            evidence_id: None,
        };
        assert_eq!(req.generated_unique_award_id, "AW-001");
        assert!(req.obligation_amount.is_none());
    }
}
