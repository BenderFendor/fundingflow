use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::Row;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{ImportSummary, Importer};

const BASE_URL: &str = "https://api.open.fec.gov/v1";
#[allow(dead_code)]
const PAGE_SIZE: i64 = 100;

pub struct OpenFecClient {
    client: reqwest::Client,
    api_key: String,
    last_request: std::sync::Mutex<Option<Instant>>,
}

impl OpenFecClient {
    pub fn new(api_key: Option<String>) -> Self {
        let key = api_key.unwrap_or_else(|| {
            std::env::var("FEC_API_KEY").unwrap_or_else(|_| "DEMO_KEY".to_string())
        });
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .user_agent("FundingFlow/0.1 (public-record research)")
                .build()
                .unwrap(),
            api_key: key,
            last_request: std::sync::Mutex::new(None),
        }
    }

    async fn rate_limit(&self) {
        let wait = {
            let guard = self.last_request.lock().unwrap();
            if let Some(last) = *guard {
                let elapsed = last.elapsed();
                if elapsed < Duration::from_millis(2100) {
                    Duration::from_millis(2100) - elapsed
                } else {
                    Duration::ZERO
                }
            } else {
                Duration::ZERO
            }
        };
        if !wait.is_zero() {
            tokio::time::sleep(wait).await;
        }
        *self.last_request.lock().unwrap() = Some(Instant::now());
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        self.rate_limit().await;
        let url = format!("{}{}", BASE_URL, path);
        let mut query = vec![("api_key", self.api_key.as_str())];
        for (k, v) in params {
            query.push((*k, *v));
        }
        let resp = self.client.get(&url).query(&query).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("OpenFEC API error: HTTP {} for {}", resp.status(), url);
        }
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Deserialize)]
struct PaginatedResponse<T> {
    pagination: Pagination,
    results: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    #[allow(dead_code)]
    count: i64,
    #[allow(dead_code)]
    page: i64,
    pages: i64,
}

#[derive(Debug, Deserialize)]
struct ScheduleEResult {
    committee_id: Option<String>,
    #[allow(dead_code)]
    committee_name: Option<String>,
    candidate_id: Option<String>,
    candidate_name: Option<String>,
    payee_name: Option<String>,
    expenditure_amount: Option<f64>,
    expenditure_date: Option<String>,
    support_oppose_indicator: Option<String>,
    memo_text: Option<String>,
    sub_id: Option<String>,
}

pub struct ScheduleEImporter {
    client: OpenFecClient,
    cycle: i32,
}

impl ScheduleEImporter {
    pub fn new(cycle: i32) -> Self {
        Self {
            client: OpenFecClient::new(None),
            cycle,
        }
    }
}

#[async_trait]
impl Importer for ScheduleEImporter {
    fn source_name(&self) -> &'static str {
        "openfec-schedule-e"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        let mut summary = ImportSummary {
            source: "openfec-schedule-e".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let cmte_map = load_fec_committee_map(pool).await;
        info!(
            "Schedule E: loaded {} committee entities from DB",
            cmte_map.len()
        );

        let cycle_str = self.cycle.to_string();
        let mut page = 1i64;
        let mut total = 0u64;

        loop {
            let params = [
                ("two_year_transaction_period", cycle_str.as_str()),
                ("per_page", "100"),
                ("page", &page.to_string()),
                ("sort", "-expenditure_date"),
            ];

            let resp: PaginatedResponse<ScheduleEResult> = match self
                .client
                .get_json("/schedules/schedule_e/", &params)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    warn!("Schedule E: page {} failed: {e}", page);
                    summary.errors += 1;
                    break;
                }
            };

            if resp.results.is_empty() {
                break;
            }

            for item in &resp.results {
                if let Err(e) = process_schedule_e_item(pool, item, self.cycle, &cmte_map).await {
                    warn!("Schedule E: item failed: {e}");
                    summary.errors += 1;
                } else {
                    summary.records_imported += 1;
                }
            }

            total += resp.results.len() as u64;
            info!(
                "Schedule E: page {}/{}, {} records so far",
                page, resp.pagination.pages, total
            );

            if page >= resp.pagination.pages {
                break;
            }
            page += 1;
        }

        info!(
            "Schedule E import done: {} records, {} errors",
            summary.records_imported, summary.errors
        );

        Ok(summary)
    }
}

async fn process_schedule_e_item(
    pool: &PgPool,
    item: &ScheduleEResult,
    cycle: i32,
    cmte_map: &HashMap<String, Uuid>,
) -> Result<()> {
    let sub_id = item.sub_id.as_deref().unwrap_or("");
    if sub_id.is_empty() {
        return Ok(());
    }

    let cmte_fec_id = item.committee_id.as_deref().unwrap_or("");
    let cmte_entity_id = cmte_map.get(cmte_fec_id).copied();

    let amount = item.expenditure_amount.unwrap_or(0.0);
    let date = item
        .expenditure_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let memo = item.memo_text.as_deref().unwrap_or("");
    let support_oppose = item.support_oppose_indicator.as_deref().unwrap_or("");
    let candidate_name = item.candidate_name.as_deref().unwrap_or("");
    let payee = item.payee_name.as_deref().unwrap_or("");

    let action = if support_oppose == "S" {
        "supporting"
    } else {
        "opposing"
    };
    let description = format!(
        "Independent expenditure {} {}: {} (payee: {})",
        action, candidate_name, memo, payee
    );

    sqlx::query(
        r#"INSERT INTO campaign_finance_transaction
           (transaction_id, sub_id, committee_entity_id, committee_fec_id,
            candidate_fec_id, amount, date, memo_text, transaction_type,
            transaction_type_code, cycle, evidence_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'independent_expenditure', '24/51', $9, NULL)
           ON CONFLICT (sub_id) DO NOTHING"#,
    )
    .bind(sub_id)
    .bind(sub_id)
    .bind(cmte_entity_id)
    .bind(cmte_fec_id)
    .bind(item.candidate_id.as_deref().unwrap_or(""))
    .bind(amount)
    .bind(date)
    .bind(memo)
    .bind(cycle)
    .execute(pool)
    .await
    .ok();

    if let (Some(from_id), Some(cand_fec_id)) = (cmte_entity_id, item.candidate_id.as_deref())
        && let Ok(Some(cand_row)) =
            sqlx::query("SELECT entity_id FROM candidate_metadata WHERE candidate_fec_id = $1")
                .bind(cand_fec_id)
                .fetch_optional(pool)
                .await
    {
        let to_id: Uuid = cand_row.get("entity_id");
        sqlx::query(
                r#"INSERT INTO relationship_edge
                   (from_entity_id, to_entity_id, edge_type, amount, currency,
                    description, confidence, source, started_on)
                   VALUES ($1, $2, 'committee_contributed_to_candidate'::edge_type, $3, 'USD', $4, 1.0, 'openfec', $5)"#,
            )
            .bind(from_id)
            .bind(to_id)
            .bind(amount)
            .bind(&description)
            .bind(date)
            .execute(pool)
            .await
            .ok();
    }

    Ok(())
}

async fn load_fec_committee_map(pool: &PgPool) -> HashMap<String, Uuid> {
    let rows = sqlx::query(
        "SELECT identifier_value, entity_id FROM entity_identifier WHERE identifier_type = 'fec_committee_id'::identifier_type",
    )
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| (r.get("identifier_value"), r.get("entity_id")))
            .collect(),
        Err(_) => HashMap::new(),
    }
}

#[allow(dead_code)]
pub async fn search_committees_api(
    client: &OpenFecClient,
    name: &str,
) -> Result<Vec<(String, String)>> {
    let params = [("q", name), ("per_page", "20")];
    let resp: PaginatedResponse<CommitteeSearchResult> =
        client.get_json("/committees/", &params).await?;

    Ok(resp
        .results
        .iter()
        .filter_map(|r| {
            r.committee_id
                .as_ref()
                .map(|id| (id.clone(), r.name.clone().unwrap_or_default()))
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct CommitteeSearchResult {
    committee_id: Option<String>,
    name: Option<String>,
}
