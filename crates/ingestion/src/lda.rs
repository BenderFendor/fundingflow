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
struct LdaResponse<T> {
    count: Option<i64>,
    results: Vec<T>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaFiling {
    filing_uuid: String,
    filing_year: Option<i32>,
    filing_period: Option<String>,
    filing_period_display: Option<String>,
    filing_type: Option<String>,
    filing_type_display: Option<String>,
    income: Option<f64>,
    expenses: Option<f64>,
    client: Option<LdaClientRef>,
    registrant: Option<LdaRegistrantRef>,
    lobbying_activities: Option<Vec<LdaActivity>>,
    dt_posted: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaClientRef {
    id: Option<i64>,
    name: Option<String>,
    state: Option<String>,
    country: Option<String>,
    general_description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaRegistrantRef {
    id: Option<i64>,
    name: Option<String>,
    city: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaActivity {
    description: Option<String>,
    general_issue_code: Option<String>,
    general_issue_code_display: Option<String>,
    lobbyists: Option<Vec<LdaLobbyistRef>>,
    government_entities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaLobbyistRef {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LdaContribution {
    id: Option<i64>,
    registrant_name: Option<String>,
    registrant_id: Option<i64>,
    client_name: Option<String>,
    client_id: Option<i64>,
    lobbyist_name: Option<String>,
    lobbyist_id: Option<i64>,
    recipient_name: Option<String>,
    contribution_date: Option<String>,
    amount: Option<f64>,
    honoree_name: Option<String>,
    payee_name: Option<String>,
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

        let params = [("filing_year", "2025"), ("page", "1"), ("page_size", "20")];

        let response = self.client.get(&url).query(&params).send().await?;

        if !response.status().is_success() {
            warn!("LDA API returned status {}", response.status());
            summary.errors += 1;
            return Ok(summary);
        }

        let body_text = response.text().await?;
        let hash = content_hash(&body_text);

        let data: LdaResponse<LdaFiling> = serde_json::from_str(&body_text)?;

        info!(
            "LDA: got {} filings (of {})",
            data.results.len(),
            data.count.unwrap_or(0),
        );

        for filing in &data.results {
            let filing_id = &filing.filing_uuid;

            let source_record = match queries::create_source_record(
                pool,
                "lda",
                "filing",
                filing_id,
                Some(&format!("{}/api/v1/filings/{}/", self.base_url, filing_id)),
                &hash,
                None,
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("lda source record error: {e}");
                    summary.errors += 1;
                    continue;
                }
            };

            let evidence = match queries::create_evidence(
                pool,
                source_record.id,
                Some("$"),
                Some(&format!("{}/api/v1/filings/{}/", self.base_url, filing_id)),
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
            if let Some(client_ref) = &filing.client {
                if let (Some(id), Some(name)) = (client_ref.id, &client_ref.name) {
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
            if let Some(reg_ref) = &filing.registrant {
                if let (Some(id), Some(name)) = (reg_ref.id, &reg_ref.name) {
                    match Self::find_or_create_registrant_entity(pool, id, name).await {
                        Ok(eid) => {
                            registrant_entity_id = Some(eid);
                            summary.entities_created += 1;
                        }
                        Err(e) => warn!("lda registrant entity error: {e}"),
                    }
                }
            }

            let amount = filing.income.or(filing.expenses);

            let issues_json = filing.lobbying_activities.as_ref().map(|activities| {
                serde_json::json!({
                    "count": activities.len(),
                    "issues": activities.iter().map(|a| {
                        serde_json::json!({
                            "code": a.general_issue_code,
                            "description": a.description,
                            "lobbyist_count": a.lobbyists.as_ref().map(|l| l.len()).unwrap_or(0),
                        })
                    }).collect::<Vec<_>>(),
                })
            });

            if let Err(e) = queries::create_lobbying_filing(
                pool,
                &CreateLobbyingFilingRequest {
                    filing_uuid: filing_id.clone(),
                    registrant_entity_id,
                    client_entity_id,
                    filing_type: filing.filing_type.clone(),
                    filing_period: filing.filing_period.clone(),
                    filing_year: filing.filing_year,
                    income_or_expense: if filing.income.is_some() {
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
                warn!("lda filing insert error: {e}");
                summary.errors += 1;
                continue;
            }

            if let (Some(client_id), Some(registrant_id)) = (client_entity_id, registrant_entity_id)
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
                        filing.filing_year.unwrap_or(0),
                        filing.filing_period_display.as_deref().unwrap_or("unknown")
                    )),
                    confidence: Some(1.0),
                    evidence_id: Some(evidence.id),
                    source: "lda".to_string(),
                };
                let _ = queries::create_relationship_edge(pool, &edge_req).await;
            }

            summary.records_imported += 1;
        }

        info!(
            "LDA import done: {} imported, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}
