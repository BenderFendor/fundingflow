use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use db::queries;
use domain::*;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::{ImportSummary, Importer, content_hash, normalize_name};

#[derive(Debug, Deserialize)]
struct FederalRegisterResponse {
    count: Option<i64>,
    results: Option<Vec<FederalRegisterDocument>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FederalRegisterDocument {
    document_number: Option<String>,
    title: Option<String>,
    #[serde(rename = "type")]
    document_type: Option<String>,
    publication_date: Option<String>,
    agencies: Option<Vec<AgencyRef>>,
    docket_ids: Option<Vec<String>>,
    abstract_: Option<String>,
    html_url: Option<String>,
    comments_close_on: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgencyRef {
    name: Option<String>,
    slug: Option<String>,
}

pub struct RulemakingImporter {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl RulemakingImporter {
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
}

#[async_trait]
impl Importer for RulemakingImporter {
    fn source_name(&self) -> &'static str {
        "rulemaking"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("Rulemaking import starting");

        let mut summary = ImportSummary {
            source: "rulemaking".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let url = format!("{}/api/v1/documents.json", self.base_url);
        let params = [
            ("per_page", "20"),
            ("order", "newest"),
            ("conditions[type][]", "RULE"),
        ];

        let resp = match self.client.get(&url).query(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("Federal Register API error: {e}");
                summary.errors += 1;
                return Ok(summary);
            }
        };

        let body_text = resp.text().await?;
        let hash = content_hash(&body_text);

        let data: FederalRegisterResponse = match serde_json::from_str(&body_text) {
            Ok(d) => d,
            Err(e) => {
                warn!("Federal Register parse error: {e}");
                summary.errors += 1;
                return Ok(summary);
            }
        };

        let results = data.results.unwrap_or_default();

        info!(
            "Federal Register: {} documents (of {})",
            results.len(),
            data.count.unwrap_or(0),
        );

        for doc in &results {
            let doc_number = doc.document_number.as_deref().unwrap_or("unknown");

            let source_record = match queries::create_source_record(
                pool,
                "federal_register",
                "document",
                doc_number,
                doc.html_url.as_deref(),
                &hash,
                None,
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("rulemaking source record error: {e}");
                    summary.errors += 1;
                    continue;
                }
            };

            let evidence = queries::create_evidence(
                pool,
                source_record.id,
                Some("$"),
                doc.html_url.as_deref(),
                Some(1.0),
            )
            .await?;

            let _publication_date = doc
                .publication_date
                .as_deref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

            let _comments_close = doc
                .comments_close_on
                .as_deref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

            if let Some(agencies) = &doc.agencies {
                for agency in agencies.iter().take(3) {
                    let agency_name = agency.name.as_deref().unwrap_or("Unknown Agency");

                    let normalized = normalize_name(agency_name);
                    let agency_entity_id = match queries::find_entity_by_name(pool, &normalized)
                        .await
                        .ok()
                        .flatten()
                    {
                        Some(e) => e.id,
                        None => {
                            match queries::create_entity(
                                pool,
                                &EntityType::Agency,
                                agency_name,
                                &normalized,
                            )
                            .await
                            {
                                Ok(e) => {
                                    summary.entities_created += 1;
                                    e.id
                                }
                                Err(e) => {
                                    warn!("rulemaking agency entity error: {e}");
                                    continue;
                                }
                            }
                        }
                    };

                    if let Some(slug) = &agency.slug {
                        let _ = queries::create_entity_identifier(
                            pool,
                            agency_entity_id,
                            &IdentifierSource::FederalRegister,
                            &IdentifierType::AgencySlug,
                            slug,
                            Some(evidence.id),
                        )
                        .await;
                    }

                    let _ = queries::create_relationship_edge(
                        pool,
                        &CreateEdgeRequest {
                            from_entity_id: agency_entity_id,
                            to_entity_id: agency_entity_id,
                            edge_type: EdgeType::AgencyPublishedRulemakingDocument,
                            started_on: _publication_date,
                            ended_on: None,
                            amount: None,
                            currency: None,
                            description: doc.title.clone(),
                            confidence: Some(1.0),
                            evidence_id: Some(evidence.id),
                            source: "federal_register".into(),
                        },
                    )
                    .await;
                }
            }

            summary.records_imported += 1;
        }

        info!(
            "Rulemaking import done: {} imported, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}
