use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use db::queries;
use domain::*;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{ImportSummary, Importer, content_hash, normalize_name};

#[derive(Debug, Deserialize)]
struct SpendingByAwardResponse {
    results: Vec<AwardResult>,
    page_metadata: PageMetadata,
}

#[derive(Debug, Deserialize)]
struct PageMetadata {
    #[allow(dead_code)]
    page: i64,
    #[serde(rename = "hasNext")]
    _has_next: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AwardResult {
    #[serde(rename = "Award ID")]
    award_id: Option<serde_json::Value>,
    #[serde(rename = "Recipient Name")]
    recipient_name: Option<String>,
    #[serde(rename = "Recipient UEI")]
    recipient_uei: Option<String>,
    #[serde(rename = "Awarding Agency")]
    awarding_agency: Option<String>,
    #[serde(rename = "Funding Agency")]
    _funding_agency: Option<String>,
    #[serde(rename = "Award Amount")]
    award_amount: Option<f64>,
    #[serde(rename = "Action Date")]
    action_date: Option<String>,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "Award Type")]
    award_type: Option<String>,
    #[serde(rename = "Place of Performance City")]
    place_of_performance_city: Option<String>,
    #[serde(rename = "Place of Performance State")]
    place_of_performance_state: Option<String>,
}

pub struct UsaspendingImporter {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl UsaspendingImporter {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("FundingFlow/0.1 (public-record research)")
                .build()
                .unwrap(),
        }
    }

    fn extract_award_id(val: &Option<serde_json::Value>) -> String {
        match val {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(v) => v.to_string(),
            None => "unknown".to_string(),
        }
    }

    fn parse_date(date_str: &Option<String>) -> Option<chrono::NaiveDate> {
        date_str
            .as_ref()
            .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    async fn find_or_create_entity(
        pool: &PgPool,
        name: &str,
        entity_type: EntityType,
        uei: Option<&str>,
    ) -> Result<Uuid> {
        if let Some(uei_val) = uei
            && !uei_val.is_empty()
            && let Ok(Some(existing)) = queries::find_entity_by_identifier(
                pool,
                &IdentifierSource::Usaspending,
                &IdentifierType::Uei,
                uei_val,
            )
            .await
        {
            return Ok(existing.id);
        }

        let normalized = normalize_name(name);
        if let Ok(Some(existing)) = queries::find_entity_by_name(pool, &normalized).await {
            if let Some(uei_val) = uei
                && !uei_val.is_empty()
            {
                let _ = queries::create_entity_identifier(
                    pool,
                    existing.id,
                    &IdentifierSource::Usaspending,
                    &IdentifierType::Uei,
                    uei_val,
                    None,
                )
                .await;
            }
            return Ok(existing.id);
        }

        let entity = queries::create_entity(pool, &entity_type, name, &normalized).await?;

        if let Some(uei_val) = uei
            && !uei_val.is_empty()
        {
            let _ = queries::create_entity_identifier(
                pool,
                entity.id,
                &IdentifierSource::Usaspending,
                &IdentifierType::Uei,
                uei_val,
                None,
            )
            .await;
        }

        Ok(entity.id)
    }

    fn build_place(city: &Option<String>, state: &Option<String>) -> Option<String> {
        match (city, state) {
            (Some(c), Some(s)) => Some(format!("{}, {}", c, s)),
            (Some(c), None) => Some(c.clone()),
            _ => None,
        }
    }

    fn state_name_to_code(name: &str) -> Option<String> {
        let name = name.trim();
        if name.len() == 2 && name.chars().all(|c| c.is_ascii_uppercase()) {
            return Some(name.to_string());
        }
        match name.to_lowercase().as_str() {
            "alabama" => Some("AL".into()),
            "alaska" => Some("AK".into()),
            "arizona" => Some("AZ".into()),
            "arkansas" => Some("AR".into()),
            "california" => Some("CA".into()),
            "colorado" => Some("CO".into()),
            "connecticut" => Some("CT".into()),
            "delaware" => Some("DE".into()),
            "district of columbia" => Some("DC".into()),
            "florida" => Some("FL".into()),
            "georgia" => Some("GA".into()),
            "hawaii" => Some("HI".into()),
            "idaho" => Some("ID".into()),
            "illinois" => Some("IL".into()),
            "indiana" => Some("IN".into()),
            "iowa" => Some("IA".into()),
            "kansas" => Some("KS".into()),
            "kentucky" => Some("KY".into()),
            "louisiana" => Some("LA".into()),
            "maine" => Some("ME".into()),
            "maryland" => Some("MD".into()),
            "massachusetts" => Some("MA".into()),
            "michigan" => Some("MI".into()),
            "minnesota" => Some("MN".into()),
            "mississippi" => Some("MS".into()),
            "missouri" => Some("MO".into()),
            "montana" => Some("MT".into()),
            "nebraska" => Some("NE".into()),
            "nevada" => Some("NV".into()),
            "new hampshire" => Some("NH".into()),
            "new jersey" => Some("NJ".into()),
            "new mexico" => Some("NM".into()),
            "new york" => Some("NY".into()),
            "north carolina" => Some("NC".into()),
            "north dakota" => Some("ND".into()),
            "ohio" => Some("OH".into()),
            "oklahoma" => Some("OK".into()),
            "oregon" => Some("OR".into()),
            "pennsylvania" => Some("PA".into()),
            "rhode island" => Some("RI".into()),
            "south carolina" => Some("SC".into()),
            "south dakota" => Some("SD".into()),
            "tennessee" => Some("TN".into()),
            "texas" => Some("TX".into()),
            "utah" => Some("UT".into()),
            "vermont" => Some("VT".into()),
            "virginia" => Some("VA".into()),
            "washington" => Some("WA".into()),
            "west virginia" => Some("WV".into()),
            "wisconsin" => Some("WI".into()),
            "wyoming" => Some("WY".into()),
            _ => None,
        }
    }
}

#[async_trait]
impl Importer for UsaspendingImporter {
    fn source_name(&self) -> &'static str {
        "usaspending"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("USASpending import starting");

        let mut summary = ImportSummary {
            source: "usaspending".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let url = format!("{}/api/v2/search/spending_by_award/", self.base_url);
        let limit = 100;
        let mut page = 1;

        loop {
            let body = serde_json::json!({
                "filters": {
                    "award_type_codes": ["A", "B", "C", "D"],
                    "time_period": [{"start_date": "2025-01-01", "end_date": "2025-12-31"}]
                },
                "fields": [
                    "Award ID", "Recipient Name", "Recipient UEI",
                    "Awarding Agency", "Funding Agency", "Award Amount",
                    "Action Date", "Description", "Award Type",
                    "Place of Performance City", "Place of Performance State"
                ],
                "sort": "Award Amount",
                "order": "desc",
                "page": page,
                "limit": limit
            });

            let content_str = serde_json::to_string(&body)?;
            let hash = content_hash(&content_str);

            let response = match self.client.post(&url).json(&body).send().await {
                Ok(r) => r,
                Err(e) => {
                    warn!("USASpending API request error on page {page}: {e}");
                    summary.errors += 1;
                    break;
                }
            };

            if !response.status().is_success() {
                warn!(
                    "USASpending API returned status {} on page {page}",
                    response.status()
                );
                summary.errors += 1;
                break;
            }

            let data: SpendingByAwardResponse = match response.json().await {
                Ok(d) => d,
                Err(e) => {
                    warn!("USASpending JSON parse error on page {page}: {e}");
                    summary.errors += 1;
                    break;
                }
            };

            let result_count = data.results.len();
            info!("USASpending: page {page} with {result_count} results",);

            if result_count == 0 {
                break;
            }

            for award in &data.results {
                let award_id_str = Self::extract_award_id(&award.award_id);

                let source_record = match queries::upsert_source_record(
                    pool,
                    "usaspending",
                    "award",
                    &award_id_str,
                    Some(&format!(
                        "{}/api/v2/awards/{}/",
                        self.base_url, award_id_str
                    )),
                    &hash,
                    None,
                )
                .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("source record upsert error: {e}");
                        summary.errors += 1;
                        continue;
                    }
                };

                let recipient_name = award
                    .recipient_name
                    .as_deref()
                    .unwrap_or("Unknown Recipient");

                let recipient_id = match Self::find_or_create_entity(
                    pool,
                    recipient_name,
                    EntityType::Organization,
                    award.recipient_uei.as_deref(),
                )
                .await
                {
                    Ok(id) => id,
                    Err(e) => {
                        warn!("recipient entity error: {e}");
                        summary.errors += 1;
                        continue;
                    }
                };

                let agency_name = award.awarding_agency.as_deref().unwrap_or("Unknown Agency");

                let agency_id =
                    match Self::find_or_create_entity(pool, agency_name, EntityType::Agency, None)
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            warn!("agency entity error: {e}");
                            summary.errors += 1;
                            continue;
                        }
                    };

                let amount = award.award_amount.unwrap_or(0.0);
                let action_date = Self::parse_date(&award.action_date);

                let evidence = match queries::create_evidence(
                    pool,
                    source_record.id,
                    Some("$"),
                    Some(&format!(
                        "{}/api/v2/awards/{}/",
                        self.base_url, award_id_str
                    )),
                    Some(1.0),
                )
                .await
                {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("evidence error: {e}");
                        summary.errors += 1;
                        continue;
                    }
                };

                let place = Self::build_place(
                    &award.place_of_performance_city,
                    &award.place_of_performance_state,
                );

                let place_geo_id = award
                    .place_of_performance_state
                    .as_deref()
                    .filter(|s| !s.is_empty() && *s != "None")
                    .and_then(Self::state_name_to_code);

                if let Err(e) = queries::upsert_award(
                    pool,
                    &CreateAwardRequest {
                        generated_unique_award_id: award_id_str.clone(),
                        recipient_entity_id: Some(recipient_id),
                        awarding_agency_entity_id: Some(agency_id),
                        funding_agency_entity_id: None,
                        award_type: award.award_type.clone(),
                        description: award.description.clone(),
                        period_start: action_date,
                        period_end: None,
                        obligation_amount: Some(amount),
                        outlay_amount: None,
                        place_of_performance: place,
                        recipient_geo_id: None,
                        place_geo_id,
                        evidence_id: Some(evidence.id),
                    },
                )
                .await
                {
                    warn!("award insert error: {e}");
                    summary.errors += 1;
                    continue;
                }

                let edge_req = CreateEdgeRequest {
                    from_entity_id: agency_id,
                    to_entity_id: recipient_id,
                    edge_type: EdgeType::AgencyObligatedAwardToRecipient,
                    started_on: action_date,
                    ended_on: None,
                    amount: Some(amount),
                    currency: Some("USD".to_string()),
                    description: award.description.clone(),
                    confidence: Some(1.0),
                    evidence_id: Some(evidence.id),
                    source: "usaspending".to_string(),
                };
                let _ = queries::create_relationship_edge(pool, &edge_req).await;

                let recip_edge = CreateEdgeRequest {
                    from_entity_id: recipient_id,
                    to_entity_id: agency_id,
                    edge_type: EdgeType::RecipientReceivedFederalObligation,
                    started_on: action_date,
                    ended_on: None,
                    amount: Some(amount),
                    currency: Some("USD".to_string()),
                    description: award.description.clone(),
                    confidence: Some(1.0),
                    evidence_id: Some(evidence.id),
                    source: "usaspending".to_string(),
                };
                let _ = queries::create_relationship_edge(pool, &recip_edge).await;

                summary.records_imported += 1;
                summary.entities_created += 2;
            }

            if !data.page_metadata._has_next {
                break;
            }
            page += 1;
        }

        info!(
            "USASpending import done: {} imported, {} errors across {} pages",
            summary.records_imported, summary.errors, page
        );

        Ok(summary)
    }
}
