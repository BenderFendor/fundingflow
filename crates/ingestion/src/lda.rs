#![allow(clippy::collapsible_if)]

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
struct LdaResponse {
    count: Option<i64>,
    results: Vec<serde_json::Value>,
}

pub struct LdaImporter {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl LdaImporter {
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

    async fn find_or_create_client_entity(pool: &PgPool, lda_id: i64, name: &str) -> Result<Uuid> {
        let id_str = lda_id.to_string();

        if let Ok(Some(existing)) = queries::find_entity_by_identifier(
            pool,
            &IdentifierSource::Lda,
            &IdentifierType::LdaClientId,
            &id_str,
        )
        .await
        {
            return Ok(existing.id);
        }

        let normalized = normalize_name(name);

        if let Ok(Some(existing)) = queries::find_entity_by_name(pool, &normalized).await {
            let _ = queries::create_entity_identifier(
                pool,
                existing.id,
                &IdentifierSource::Lda,
                &IdentifierType::LdaClientId,
                &id_str,
                None,
            )
            .await;
            return Ok(existing.id);
        }

        let entity =
            queries::create_entity(pool, &EntityType::Organization, name, &normalized).await?;
        let _ = queries::create_entity_identifier(
            pool,
            entity.id,
            &IdentifierSource::Lda,
            &IdentifierType::LdaClientId,
            &id_str,
            None,
        )
        .await;
        Ok(entity.id)
    }

    async fn find_or_create_registrant_entity(
        pool: &PgPool,
        lda_id: i64,
        name: &str,
    ) -> Result<Uuid> {
        let id_str = lda_id.to_string();

        if let Ok(Some(existing)) = queries::find_entity_by_identifier(
            pool,
            &IdentifierSource::Lda,
            &IdentifierType::LdaRegistrantId,
            &id_str,
        )
        .await
        {
            return Ok(existing.id);
        }

        let normalized = normalize_name(name);

        if let Ok(Some(existing)) = queries::find_entity_by_name(pool, &normalized).await {
            let _ = queries::create_entity_identifier(
                pool,
                existing.id,
                &IdentifierSource::Lda,
                &IdentifierType::LdaRegistrantId,
                &id_str,
                None,
            )
            .await;
            return Ok(existing.id);
        }

        let entity =
            queries::create_entity(pool, &EntityType::Organization, name, &normalized).await?;
        let _ = queries::create_entity_identifier(
            pool,
            entity.id,
            &IdentifierSource::Lda,
            &IdentifierType::LdaRegistrantId,
            &id_str,
            None,
        )
        .await;
        Ok(entity.id)
    }
}

#[async_trait]
impl Importer for LdaImporter {
    fn source_name(&self) -> &'static str {
        "lda"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("LDA import starting");

        let mut summary = ImportSummary {
            source: "lda".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let url = format!("{}/api/v1/filings/", self.base_url);
        let filing_years = ["2024", "2025"];
        let page_size = 100;

        for year in &filing_years {
            let mut page = 1;
            let mut backoff_ms = 200u64;

            loop {
                tokio::time::sleep(Duration::from_millis(backoff_ms.min(5000))).await;

                let params = [
                    ("filing_year", *year),
                    ("page", &page.to_string()),
                    ("page_size", &page_size.to_string()),
                ];

                let response = match self.client.get(&url).query(&params).send().await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("LDA API request error page {page} year {year}: {e}");
                        summary.errors += 1;
                        break;
                    }
                };

                if response.status().as_u16() == 429 {
                    warn!(
                        "LDA rate limited page {page} year {year}, backing off {}ms",
                        backoff_ms
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(30000);
                    continue;
                }

                backoff_ms = 200;

                if !response.status().is_success() {
                    warn!(
                        "LDA API returned status {} page {page} year {year}",
                        response.status()
                    );
                    summary.errors += 1;
                    break;
                }

                let body_text = match response.text().await {
                    Ok(t) => t,
                    Err(e) => {
                        warn!("LDA body read error: {e}");
                        summary.errors += 1;
                        break;
                    }
                };
                let hash = content_hash(&body_text);

                let data: LdaResponse = match serde_json::from_str(&body_text) {
                    Ok(d) => d,
                    Err(e) => {
                        warn!("LDA JSON parse error page {page} year {year}: {e}");
                        summary.errors += 1;
                        page += 1;
                        continue;
                    }
                };

                let result_count = data.results.len();
                info!(
                    "LDA: year={year} page={page} got {result_count} filings (of {})",
                    data.count.unwrap_or(0),
                );

                if result_count == 0 {
                    break;
                }

                for filing_value in &data.results {
                    let filing_uuid = match filing_value.get("filing_uuid").and_then(|v| v.as_str())
                    {
                        Some(u) => u.to_string(),
                        None => {
                            summary.errors += 1;
                            continue;
                        }
                    };

                    let filing_year = filing_value
                        .get("filing_year")
                        .and_then(|v| v.as_i64())
                        .map(|n| n as i32);
                    let filing_period = filing_value
                        .get("filing_period")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let filing_type = filing_value
                        .get("filing_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let filing_period_display = filing_value
                        .get("filing_period_display")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let income = filing_value.get("income").and_then(|v| v.as_f64());
                    let expenses = filing_value.get("expenses").and_then(|v| v.as_f64());

                    let source_record = match queries::upsert_source_record(
                        pool,
                        "lda",
                        "filing",
                        &filing_uuid,
                        Some(&format!(
                            "{}/api/v1/filings/{}/",
                            self.base_url, filing_uuid
                        )),
                        &hash,
                        None,
                    )
                    .await
                    {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("lda source record upsert error: {e}");
                            summary.errors += 1;
                            continue;
                        }
                    };

                    let evidence = match queries::create_evidence(
                        pool,
                        source_record.id,
                        Some("$"),
                        Some(&format!(
                            "{}/api/v1/filings/{}/",
                            self.base_url, filing_uuid
                        )),
                        Some(1.0),
                    )
                    .await
                    {
                        Ok(e) => e,
                        Err(e) => {
                            warn!("lda evidence error: {e}");
                            summary.errors += 1;
                            continue;
                        }
                    };

                    let mut client_entity_id = None;
                    if let Some(client_ref) = filing_value.get("client") {
                        let id = client_ref.get("id").and_then(|v| v.as_i64());
                        let name = client_ref.get("name").and_then(|v| v.as_str());
                        if let (Some(id), Some(name)) = (id, name) {
                            match Self::find_or_create_client_entity(pool, id, name).await {
                                Ok(eid) => {
                                    client_entity_id = Some(eid);
                                    summary.entities_created += 1;
                                }
                                Err(e) => warn!("lda client entity error: {e}"),
                            }
                        }
                    }

                    let mut registrant_entity_id = None;
                    if let Some(reg_ref) = filing_value.get("registrant") {
                        let id = reg_ref.get("id").and_then(|v| v.as_i64());
                        let name = reg_ref.get("name").and_then(|v| v.as_str());
                        if let (Some(id), Some(name)) = (id, name) {
                            match Self::find_or_create_registrant_entity(pool, id, name).await {
                                Ok(eid) => {
                                    registrant_entity_id = Some(eid);
                                    summary.entities_created += 1;
                                }
                                Err(e) => warn!("lda registrant entity error: {e}"),
                            }
                        }
                    }

                    let amount = income.or(expenses);

                    let issues_json = filing_value.get("lobbying_activities").map(|activities| {
                        let count = activities.as_array().map(|a| a.len()).unwrap_or(0);
                        serde_json::json!({
                            "count": count,
                            "issues": activities,
                        })
                    });

                    if let Err(e) = queries::upsert_lobbying_filing(
                        pool,
                        &CreateLobbyingFilingRequest {
                            filing_uuid: filing_uuid.clone(),
                            registrant_entity_id,
                            client_entity_id,
                            filing_type,
                            filing_period,
                            filing_year,
                            income_or_expense: if income.is_some() {
                                Some("income".to_string())
                            } else {
                                Some("expense".to_string())
                            },
                            amount,
                            issues_json,
                            evidence_id: Some(evidence.id),
                        },
                    )
                    .await
                    {
                        warn!("lda filing upsert error: {e}");
                        summary.errors += 1;
                        continue;
                    }

                    if let (Some(client_id), Some(registrant_id)) =
                        (client_entity_id, registrant_entity_id)
                    {
                        let edge_req = CreateEdgeRequest {
                            from_entity_id: registrant_id,
                            to_entity_id: client_id,
                            edge_type: EdgeType::RegistrantLobbiedForClient,
                            started_on: None,
                            ended_on: None,
                            amount,
                            currency: Some("USD".to_string()),
                            description: Some(format!(
                                "LDA filing {} - {}",
                                filing_year.unwrap_or(0),
                                filing_period_display.as_deref().unwrap_or("unknown")
                            )),
                            confidence: Some(1.0),
                            evidence_id: Some(evidence.id),
                            source: "lda".to_string(),
                        };
                        let _ = queries::create_relationship_edge(pool, &edge_req).await;
                    }

                    summary.records_imported += 1;
                }

                page += 1;
            }
        }

        info!(
            "LDA import done: {} imported, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}
