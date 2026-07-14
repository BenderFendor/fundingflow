#![allow(clippy::collapsible_if)]

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
#[allow(non_snake_case, dead_code)]
struct SubmissionsResponse {
    cik: Option<String>,
    entityType: Option<String>,
    name: Option<String>,
    tickers: Option<Vec<String>>,
    filings: Option<FilingsWrapper>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FilingsWrapper {
    recent: Option<RecentFilings>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct RecentFilings {
    accessionNumber: Vec<String>,
    reportDate: Vec<String>,
    form: Vec<String>,
    primaryDocument: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct CompanyFactsResponse {
    cik: Option<i64>,
    entityName: Option<String>,
    facts: Option<serde_json::Value>,
}

pub struct SecImporter {
    pub base_url: String,
    pub client: reqwest::Client,
    max_companies: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompanyTicker {
    cik_str: u64,
    ticker: String,
    title: String,
}

impl SecImporter {
    pub fn new(base_url: impl Into<String>, limit: usize) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("FundingFlow/0.1 (public-record research) contact@example.com")
                .build()
                .unwrap(),
            max_companies: if limit == 0 { usize::MAX } else { limit },
        }
    }
}

#[async_trait]
impl Importer for SecImporter {
    fn source_name(&self) -> &'static str {
        "sec"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("SEC import starting");

        let mut summary = ImportSummary {
            source: "sec".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let tickers_url = "https://www.sec.gov/files/company_tickers.json";
        info!("SEC: fetching company tickers from {}", tickers_url);
        let resp = self
            .client
            .get(tickers_url)
            .header("User-Agent", "FundingFlow/0.1 contact@example.com")
            .send()
            .await?;

        let body = resp.text().await?;
        let companies: std::collections::HashMap<String, CompanyTicker> =
            serde_json::from_str(&body)?;

        info!("SEC: found {} registered companies", companies.len());

        let padded_cik = |raw: u64| -> String { format!("CIK{:0>10}", raw) };

        for company in companies.values().take(self.max_companies) {
            let cik_str = company.cik_str.to_string();
            let padded = padded_cik(company.cik_str);

            tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            let submissions_url = format!("{}/submissions/{padded}.json", self.base_url);

            let resp = match self
                .client
                .get(&submissions_url)
                .header("User-Agent", "FundingFlow/0.1 contact@example.com")
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("SEC fetch error for {cik_str}: {e}");
                    summary.errors += 1;
                    continue;
                }
            };

            if !resp.status().is_success() {
                warn!("SEC status {} for {cik_str}", resp.status());
                summary.errors += 1;
                continue;
            }

            let body_text = resp.text().await?;
            let hash = content_hash(&body_text);

            let _source_record = match queries::upsert_source_record(
                pool,
                "sec",
                "submissions",
                &cik_str,
                Some(&submissions_url),
                &hash,
                None,
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("sec source record error for {cik_str}: {e}");
                    summary.errors += 1;
                    continue;
                }
            };

            let submissions: SubmissionsResponse = match serde_json::from_str(&body_text) {
                Ok(s) => s,
                Err(e) => {
                    warn!("sec parse error for {cik_str}: {e}");
                    summary.errors += 1;
                    continue;
                }
            };

            let company_name = submissions
                .name
                .as_deref()
                .unwrap_or("Unknown SEC Registrant");

            let normalized = normalize_name(company_name);
            let entity = match queries::find_entity_by_name(pool, &normalized)
                .await
                .ok()
                .flatten()
            {
                Some(e) => e,
                None => match queries::create_entity(
                    pool,
                    &EntityType::Organization,
                    company_name,
                    &normalized,
                )
                .await
                {
                    Ok(e) => {
                        summary.entities_created += 1;
                        e
                    }
                    Err(e) => {
                        warn!("sec entity error for {cik_str}: {e}");
                        summary.errors += 1;
                        continue;
                    }
                },
            };

            let _ = queries::create_entity_identifier(
                pool,
                entity.id,
                &IdentifierSource::Sec,
                &IdentifierType::Cik,
                &cik_str,
                None,
            )
            .await;

            if let Some(tickers) = &submissions.tickers {
                for ticker in tickers.iter().take(3) {
                    let _ = queries::create_entity_identifier(
                        pool,
                        entity.id,
                        &IdentifierSource::Sec,
                        &IdentifierType::Ticker,
                        ticker,
                        None,
                    )
                    .await;
                }
            }

            let facts_url = format!("{}/api/xbrl/companyfacts/{padded}.json", self.base_url);

            if let Ok(resp) = self
                .client
                .get(&facts_url)
                .header("User-Agent", "FundingFlow/0.1 contact@example.com")
                .send()
                .await
            {
                if resp.status().is_success() {
                    if let Ok(body) = resp.text().await {
                        let facts_hash = content_hash(&body);
                        let fact_record = queries::upsert_source_record(
                            pool,
                            "sec",
                            "companyfacts",
                            &cik_str,
                            Some(&facts_url),
                            &facts_hash,
                            None,
                        )
                        .await
                        .ok();

                        if let Some(ref fact_rec) = fact_record {
                            let fact_evidence = queries::create_evidence(
                                pool,
                                fact_rec.id,
                                Some("$.facts"),
                                Some(&facts_url),
                                Some(1.0),
                            )
                            .await?;

                            let _ = queries::create_relationship_edge(
                                pool,
                                &CreateEdgeRequest {
                                    from_entity_id: entity.id,
                                    to_entity_id: entity.id,
                                    edge_type: EdgeType::CompanyFiledSecReport,
                                    started_on: None,
                                    ended_on: None,
                                    amount: None,
                                    currency: None,
                                    description: Some("Company has SEC EDGAR filings".into()),
                                    confidence: Some(1.0),
                                    evidence_id: Some(fact_evidence.id),
                                    source: "sec".into(),
                                },
                            )
                            .await;
                        }

                        if let Some(ref fact_rec) = fact_record {
                            if let Ok(facts) = serde_json::from_str::<CompanyFactsResponse>(&body) {
                                if let Some(facts_map) = facts.facts {
                                    if let Some(us_gaap) =
                                        facts_map.get("us-gaap").or_else(|| facts_map.get("dei"))
                                    {
                                        if let Some(revenue) = us_gaap
                                            .get("Revenues")
                                            .or_else(|| us_gaap.get("RevenueFromContractWithCustomerExcludingAssessedTax"))
                                        {
                                            if let Some(units_data) = revenue.get("units") {
                                                let fact_entries = units_data
                                                    .as_array()
                                                    .map(|a| a.iter().take(3).collect::<Vec<_>>())
                                                    .unwrap_or_default();

                                                for entry in fact_entries {
                                                    let val = entry.get("val")
                                                        .and_then(|v| v.as_f64());
                                                    let form = entry.get("form")
                                                        .and_then(|v| v.as_str());
                                                    let end = entry.get("end")
                                                        .and_then(|v| v.as_str());
                                                    let fy = entry.get("fy")
                                                        .and_then(|v| v.as_f64());
                                                    let fp = entry.get("fp")
                                                        .and_then(|v| v.as_str());

                                                    let period_start = fp.and_then(|fps| {
                                                        fy.and_then(|f: f64| {
                                                            let q: i32 = fps.parse().ok()?;
                                                            let month = (q - 1) * 3 + 1;
                                                            chrono::NaiveDate::from_ymd_opt(
                                                                f as i32, month as u32, 1,
                                                            )
                                                        })
                                                    });

                                                    let period_end = end.and_then(|e| {
                                                        chrono::NaiveDate::parse_from_str(
                                                            e, "%Y-%m-%d",
                                                        )
                                                        .ok()
                                                    });

                                                    if let Ok(fact_ev) =
                                                        queries::create_evidence(
                                                            pool,
                                                            fact_rec.id,
                                                            Some("$.facts"),
                                                            Some(&facts_url),
                                                            Some(1.0),
                                                        )
                                                        .await
                                                    {
                                                        let _ = queries::create_relationship_edge(
                                                            pool,
                                                            &CreateEdgeRequest {
                                                                from_entity_id: entity.id,
                                                                to_entity_id: entity.id,
                                                                edge_type:
                                                                    EdgeType::CompanyReportedRevenueFact,
                                                                started_on: period_start,
                                                                ended_on: period_end,
                                                                amount: val,
                                                                currency: Some("USD".into()),
                                                                description: Some(format!(
                                                                    "Revenue: {} ({} {})",
                                                                    form.unwrap_or("?"),
                                                                    fy.unwrap_or(0.0) as i32,
                                                                    fp.unwrap_or("?"),
                                                                )),
                                                                confidence: Some(1.0),
                                                                evidence_id: Some(fact_ev.id),
                                                                source: "sec".into(),
                                                            },
                                                        )
                                                        .await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            summary.records_imported += 1;
        }

        info!(
            "SEC import done: {} companies, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}
