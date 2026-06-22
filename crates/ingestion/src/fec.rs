#![allow(clippy::collapsible_if, clippy::get_first)]

use anyhow::Result;
use async_trait::async_trait;
use db::queries;
use domain::*;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::{ImportSummary, Importer, content_hash, normalize_name};

pub struct FecImporter {
    pub data_dir: String,
}

impl FecImporter {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }
}

#[async_trait]
impl Importer for FecImporter {
    fn source_name(&self) -> &'static str {
        "fec"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("FEC import: reading bulk data from {}", self.data_dir);

        let mut summary = ImportSummary {
            source: "fec".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let contrib_path = format!("{}/contributions_sample.csv", self.data_dir);
        let content = match tokio::fs::read_to_string(&contrib_path).await {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "FEC: could not read {}: {}. no data to import.",
                    contrib_path, e
                );
                return Ok(summary);
            }
        };

        let hash = content_hash(&content);

        let source_record = queries::create_source_record(
            pool,
            "fec",
            "contributions",
            &contrib_path,
            Some("https://www.fec.gov/data/browse-data/?tab=bulk-data"),
            &hash,
            Some(&contrib_path),
        )
        .await?;

        let evidence = queries::create_evidence(
            pool,
            source_record.id,
            None,
            Some("https://www.fec.gov/data/browse-data/?tab=bulk-data"),
            Some(1.0),
        )
        .await?;

        let mut lines = content.lines();
        let _header = lines.next();

        for line in lines.take(50) {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() < 11 {
                summary.errors += 1;
                continue;
            }

            let committee_name = fields.get(0).map(|s| s.trim());
            let contributor_name = fields.get(7).map(|s| s.trim());
            let employer = fields.get(10).map(|s| s.trim());
            let occupation = fields.get(11).map(|s| s.trim());
            let amount: Option<f64> = fields.get(14).and_then(|s| s.trim().parse().ok());
            let date_str = fields.get(13).map(|s| s.trim());
            let date = date_str.and_then(|s| {
                chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y")
                    .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                    .ok()
            });

            let committee_id = match committee_name {
                Some(name) if !name.is_empty() => {
                    let n = normalize_name(name);
                    match queries::find_entity_by_name(pool, &n).await.ok().flatten() {
                        Some(e) => Some(e.id),
                        None => {
                            match queries::create_entity(pool, &EntityType::Committee, name, &n)
                                .await
                            {
                                Ok(e) => {
                                    summary.entities_created += 1;
                                    Some(e.id)
                                }
                                Err(e) => {
                                    warn!("fec committee entity error: {e}");
                                    None
                                }
                            }
                        }
                    }
                }
                _ => None,
            };

            let contributor_id = match contributor_name {
                Some(name) if !name.is_empty() => {
                    let n = normalize_name(name);
                    match queries::find_entity_by_name(pool, &n).await.ok().flatten() {
                        Some(e) => Some(e.id),
                        None => {
                            match queries::create_entity(pool, &EntityType::Person, name, &n).await
                            {
                                Ok(e) => {
                                    summary.entities_created += 1;
                                    Some(e.id)
                                }
                                Err(e) => {
                                    warn!("fec contributor entity error: {e}");
                                    None
                                }
                            }
                        }
                    }
                }
                _ => None,
            };

            if let (Some(comm_id), Some(contrib_id)) = (committee_id, contributor_id) {
                let _ = queries::create_relationship_edge(
                    pool,
                    &CreateEdgeRequest {
                        from_entity_id: contrib_id,
                        to_entity_id: comm_id,
                        edge_type: EdgeType::PersonContributedToCommittee,
                        started_on: date,
                        ended_on: None,
                        amount,
                        currency: Some("USD".to_string()),
                        description: Some(format!(
                            "Employer: {} | Occupation: {}",
                            employer.unwrap_or("unknown"),
                            occupation.unwrap_or("unknown")
                        )),
                        confidence: Some(1.0),
                        evidence_id: Some(evidence.id),
                        source: "fec".to_string(),
                    },
                )
                .await;
            }

            if let (Some(comm_id), Some(employer_name)) = (committee_id, employer) {
                if !employer_name.is_empty()
                    && employer_name.to_lowercase() != "self employed"
                    && employer_name.to_lowercase() != "none"
                {
                    let emp_norm = normalize_name(employer_name);
                    let emp_id = match queries::find_entity_by_name(pool, &emp_norm)
                        .await
                        .ok()
                        .flatten()
                    {
                        Some(e) => e.id,
                        None => match queries::create_entity(
                            pool,
                            &EntityType::Organization,
                            employer_name,
                            &emp_norm,
                        )
                        .await
                        {
                            Ok(e) => {
                                summary.entities_created += 1;
                                e.id
                            }
                            Err(_) => continue,
                        },
                    };

                    let _ = queries::create_relationship_edge(
                        pool,
                        &CreateEdgeRequest {
                            from_entity_id: emp_id,
                            to_entity_id: comm_id,
                            edge_type: EdgeType::EmployerReportedOnContribution,
                            started_on: date,
                            ended_on: None,
                            amount: None,
                            currency: None,
                            description: Some("Employer listed on individual contribution".into()),
                            confidence: Some(0.5),
                            evidence_id: Some(evidence.id),
                            source: "fec".into(),
                        },
                    )
                    .await;
                }
            }

            summary.records_imported += 1;
        }

        info!(
            "FEC import done: {} imported, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}
