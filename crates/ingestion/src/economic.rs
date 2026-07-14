use anyhow::Result;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use db::queries;
use domain::*;
use regex::Regex;
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{info, warn};

use crate::{ImportSummary, Importer, content_hash};

// This should be split into smaller files too so we have it not just be like 3000+ lines

#[derive(Debug, Clone, Copy)]
pub enum EconomicSource {
    BlsLaus,
    BlsCes,
    BlsCpiPrices,
    BlsCps,
    DolMinWage,
    BeaRegional,
    CensusAcs,
    FhfaHpi,
    HudFmr,
    EiaGas,
    UsaspendingState,
}

impl EconomicSource {
    pub fn from_cli_name(name: &str) -> Option<Self> {
        match name {
            "bls-laus" => Some(Self::BlsLaus),
            "bls-ces" => Some(Self::BlsCes),
            "bls-cpi-prices" => Some(Self::BlsCpiPrices),
            "bls-cps" => Some(Self::BlsCps),
            "dol-min-wage" => Some(Self::DolMinWage),
            "bea-regional" => Some(Self::BeaRegional),
            "census-acs" => Some(Self::CensusAcs),
            "fhfa-hpi" => Some(Self::FhfaHpi),
            "hud-fmr" => Some(Self::HudFmr),
            "eia-gas" => Some(Self::EiaGas),
            "usaspending-state" => Some(Self::UsaspendingState),
            _ => None,
        }
    }

    fn source_id(self) -> &'static str {
        match self {
            Self::BlsLaus => "bls-laus",
            Self::BlsCes => "bls-ces",
            Self::BlsCpiPrices => "bls-cpi-prices",
            Self::BlsCps => "bls-cps",
            Self::DolMinWage => "dol-min-wage",
            Self::BeaRegional => "bea-regional",
            Self::CensusAcs => "census-acs",
            Self::FhfaHpi => "fhfa-hpi",
            Self::HudFmr => "hud-fmr",
            Self::EiaGas => "eia-gas",
            Self::UsaspendingState => "usaspending-state",
        }
    }

    fn data_source(self) -> DataSource {
        match self {
            Self::BlsLaus => DataSource {
                source_id: self.source_id().into(),
                agency: "Bureau of Labor Statistics".into(),
                dataset: "Local Area Unemployment Statistics".into(),
                url: "https://www.bls.gov/lau/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("monthly".into()),
                requires_api_key: false,
                notes: Some("State and local labor force and unemployment metrics.".into()),
            },
            Self::BlsCes => DataSource {
                source_id: self.source_id().into(),
                agency: "Bureau of Labor Statistics".into(),
                dataset: "Current Employment Statistics".into(),
                url: "https://www.bls.gov/ces/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("monthly".into()),
                requires_api_key: false,
                notes: Some("Payroll jobs and earnings series.".into()),
            },
            Self::BlsCpiPrices => DataSource {
                source_id: self.source_id().into(),
                agency: "Bureau of Labor Statistics".into(),
                dataset: "CPI and Average Price Data".into(),
                url: "https://www.bls.gov/cpi/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("monthly".into()),
                requires_api_key: false,
                notes: Some("Food CPI and average prices; food item geography is U.S. or broad region unless stated.".into()),
            },
            Self::BlsCps => DataSource {
                source_id: self.source_id().into(),
                agency: "Bureau of Labor Statistics".into(),
                dataset: "Current Population Survey".into(),
                url: "https://www.bls.gov/cps/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("quarterly".into()),
                requires_api_key: false,
                notes: Some("Median usual weekly earnings for full-time wage and salary workers by sex and race.".into()),
            },
            Self::DolMinWage => DataSource {
                source_id: self.source_id().into(),
                agency: "Department of Labor".into(),
                dataset: "State Minimum Wage Laws".into(),
                url: "https://www.dol.gov/agencies/whd/minimum-wage/state".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("as law changes".into()),
                requires_api_key: false,
                notes: Some("Minimum wage law with tipped, local, and scheduled-increase caveats.".into()),
            },
            Self::BeaRegional => DataSource {
                source_id: self.source_id().into(),
                agency: "Bureau of Economic Analysis".into(),
                dataset: "Regional Economic Accounts".into(),
                url: "https://apps.bea.gov/API/docs/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("monthly/quarterly/annual".into()),
                requires_api_key: true,
                notes: Some("GDP, personal income, compensation, and transfer receipts.".into()),
            },
            Self::CensusAcs => DataSource {
                source_id: self.source_id().into(),
                agency: "Census Bureau".into(),
                dataset: "American Community Survey 5-Year".into(),
                url: "https://www.census.gov/data/developers/data-sets/acs-5year.html".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("annual".into()),
                requires_api_key: true,
                notes: Some("Income, poverty, rent, housing burden, and demographic context.".into()),
            },
            Self::FhfaHpi => DataSource {
                source_id: self.source_id().into(),
                agency: "Federal Housing Finance Agency".into(),
                dataset: "House Price Index".into(),
                url: "https://www.fhfa.gov/data/hpi".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("monthly/quarterly".into()),
                requires_api_key: false,
                notes: Some("Repeat-sales home price index movement.".into()),
            },
            Self::HudFmr => DataSource {
                source_id: self.source_id().into(),
                agency: "Department of Housing and Urban Development".into(),
                dataset: "Fair Market Rents API".into(),
                url: "https://www.huduser.gov/portal/dataset/fmr-api.html".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("annual".into()),
                requires_api_key: true,
                notes: Some("Fair Market Rent benchmarks by state, metro, county, and small area where available.".into()),
            },
            Self::EiaGas => DataSource {
                source_id: self.source_id().into(),
                agency: "Energy Information Administration".into(),
                dataset: "Gasoline and Diesel Fuel Update".into(),
                url: "https://www.eia.gov/petroleum/gasdiesel/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("weekly".into()),
                requires_api_key: true,
                notes: Some("Fuel prices with EIA API pagination limits.".into()),
            },
            Self::UsaspendingState => DataSource {
                source_id: self.source_id().into(),
                agency: "USAspending.gov".into(),
                dataset: "State Federal Awards".into(),
                url: "https://api.usaspending.gov/docs/".into(),
                license: Some("Public domain".into()),
                update_frequency: Some("daily".into()),
                requires_api_key: false,
                notes: Some("Federal obligations by recipient and place of performance geography.".into()),
            },
        }
    }

    fn metrics(self) -> Vec<Metric> {
        match self {
            Self::BlsLaus => vec![metric(
                "unemployment_rate",
                "Unemployment Rate",
                "labor",
                "percent",
                self,
            )],
            Self::BlsCes => vec![
                metric(
                    "total_nonfarm_employment",
                    "Total Nonfarm Employment",
                    "labor",
                    "jobs",
                    self,
                ),
                metric(
                    "payroll_jobs_change",
                    "Payroll Jobs Change",
                    "labor",
                    "jobs",
                    self,
                ),
                metric(
                    "avg_hourly_earnings",
                    "Average Hourly Earnings",
                    "wages",
                    "usd_per_hour",
                    self,
                ),
            ],
            Self::BlsCpiPrices => vec![
                metric(
                    "food_at_home_cpi_yoy",
                    "Food At Home CPI YoY",
                    "food",
                    "percent",
                    self,
                ),
                metric(
                    "food_away_from_home_cpi_yoy",
                    "Food Away From Home CPI YoY",
                    "food",
                    "percent",
                    self,
                ),
                metric("egg_price", "Egg Price", "food", "usd_per_dozen", self),
                metric(
                    "milk_price",
                    "Whole Milk Price",
                    "food",
                    "usd_per_gallon",
                    self,
                ),
                metric(
                    "white_bread_price",
                    "White Bread Price",
                    "food",
                    "usd_per_lb",
                    self,
                ),
                metric(
                    "ground_beef_price",
                    "Ground Beef Price",
                    "food",
                    "usd_per_lb",
                    self,
                ),
                metric(
                    "chicken_breast_price",
                    "Chicken Breast Price",
                    "food",
                    "usd_per_lb",
                    self,
                ),
                metric("banana_price", "Banana Price", "food", "usd_per_lb", self),
            ],
            Self::BlsCps => vec![
                metric(
                    "median_weekly_earnings",
                    "Median Weekly Earnings",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_male",
                    "Median Weekly Earnings (Men)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_female",
                    "Median Weekly Earnings (Women)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_white",
                    "Median Weekly Earnings (White)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_black",
                    "Median Weekly Earnings (Black)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_asian",
                    "Median Weekly Earnings (Asian)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
                metric(
                    "median_weekly_earnings_hispanic",
                    "Median Weekly Earnings (Hispanic)",
                    "wages",
                    "usd_per_week",
                    self,
                ),
            ],
            Self::DolMinWage => vec![
                metric(
                    "state_min_wage",
                    "State Minimum Wage",
                    "wages",
                    "usd_per_hour",
                    self,
                ),
                metric(
                    "effective_min_wage",
                    "Effective Minimum Wage",
                    "wages",
                    "usd_per_hour",
                    self,
                ),
            ],
            Self::BeaRegional => vec![
                metric("state_gdp", "State GDP", "economy", "usd", self),
                metric("personal_income", "Personal Income", "income", "usd", self),
            ],
            Self::CensusAcs => vec![
                metric(
                    "median_household_income",
                    "Median Household Income",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "median_gross_rent",
                    "Median Gross Rent",
                    "housing",
                    "usd_per_month",
                    self,
                ),
                metric(
                    "rent_burden_rate",
                    "Rent Burden Rate",
                    "housing",
                    "percent",
                    self,
                ),
                metric("poverty_rate", "Poverty Rate", "income", "percent", self),
                metric(
                    "quintile_income_bottom",
                    "Income Quintile - Lowest",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "quintile_income_second",
                    "Income Quintile - Second",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "quintile_income_third",
                    "Income Quintile - Third",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "quintile_income_fourth",
                    "Income Quintile - Fourth",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "quintile_income_top",
                    "Income Quintile - Highest",
                    "income",
                    "usd",
                    self,
                ),
                metric(
                    "median_earnings_male",
                    "Median Earnings (Men)",
                    "wages",
                    "usd",
                    self,
                ),
                metric(
                    "median_earnings_female",
                    "Median Earnings (Women)",
                    "wages",
                    "usd",
                    self,
                ),
            ],
            Self::FhfaHpi => vec![metric(
                "fhfa_hpi_yoy",
                "FHFA HPI YoY",
                "housing",
                "percent",
                self,
            )],
            Self::HudFmr => vec![metric(
                "fmr_2br",
                "Two Bedroom Fair Market Rent",
                "housing",
                "usd_per_month",
                self,
            )],
            Self::EiaGas => vec![metric(
                "regular_gas_price",
                "Regular Gasoline Price",
                "energy",
                "usd_per_gallon",
                self,
            )],
            Self::UsaspendingState => vec![metric(
                "federal_contract_obligations",
                "Federal Award Obligations",
                "contracts",
                "usd",
                self,
            )],
        }
    }
}

fn metric(
    metric_id: &str,
    name: &str,
    category: &str,
    unit: &str,
    source: EconomicSource,
) -> Metric {
    Metric {
        metric_id: metric_id.into(),
        name: name.into(),
        category: category.into(),
        unit: unit.into(),
        seasonal_adjustment: None,
        source_id: source.source_id().into(),
        notes: None,
    }
}

pub struct EconomicImporter {
    source: EconomicSource,
}

impl EconomicImporter {
    pub fn new(source: EconomicSource) -> Self {
        Self { source }
    }
}

#[async_trait]
impl Importer for EconomicImporter {
    fn source_name(&self) -> &'static str {
        self.source.source_id()
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        info!("economic import starting: {}", self.source.source_id());
        ensure_base_economic_catalog(pool).await?;

        let data_source = self.source.data_source();
        queries::upsert_data_source(pool, &data_source).await?;
        for metric in self.source.metrics() {
            queries::upsert_metric(pool, &metric).await?;
        }

        let mut summary = ImportSummary {
            source: self.source.source_id().into(),
            records_imported: 1,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        if matches!(
            self.source,
            EconomicSource::BeaRegional
                | EconomicSource::CensusAcs
                | EconomicSource::HudFmr
                | EconomicSource::EiaGas
        ) && required_key_missing(self.source)
        {
            summary.records_skipped += 1;
            info!(
                "economic import skipped live observations for {} because required API key is missing",
                self.source.source_id()
            );
            return Ok(summary);
        }

        if matches!(self.source, EconomicSource::BlsLaus) {
            return import_bls_laus_state_unemployment(pool).await;
        }

        if matches!(self.source, EconomicSource::BlsCes) {
            return import_bls_ces_national_metrics(pool).await;
        }

        if matches!(self.source, EconomicSource::BlsCpiPrices) {
            return import_bls_cpi_and_average_prices(pool).await;
        }

        if matches!(self.source, EconomicSource::DolMinWage) {
            return import_dol_minimum_wages(pool).await;
        }

        if matches!(self.source, EconomicSource::UsaspendingState) {
            return import_usaspending_state_totals(pool).await;
        }

        if matches!(self.source, EconomicSource::FhfaHpi) {
            return import_fhfa_state_hpi_yoy(pool).await;
        }

        if matches!(self.source, EconomicSource::CensusAcs) {
            return import_census_acs_state_metrics(pool).await;
        }

        if matches!(self.source, EconomicSource::HudFmr) {
            return import_hud_fmr_state_rents(pool).await;
        }

        if matches!(self.source, EconomicSource::EiaGas) {
            return import_eia_gas_prices(pool).await;
        }

        if matches!(self.source, EconomicSource::BeaRegional) {
            return import_bea_regional_economic_accounts(pool).await;
        }

        if matches!(self.source, EconomicSource::BlsCps) {
            return import_bls_cps_median_earnings(pool).await;
        }

        Ok(summary)
    }
}

#[derive(Debug, Deserialize)]
struct BlsResponse {
    #[serde(rename = "Results")]
    results: Option<BlsResults>,
    status: Option<String>,
    message: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct BlsResults {
    series: Vec<BlsSeries>,
}

#[derive(Debug, Deserialize)]
struct BlsSeries {
    #[serde(rename = "seriesID")]
    series_id: String,
    data: Vec<BlsObservation>,
}

#[derive(Debug, Deserialize)]
struct BlsObservation {
    year: String,
    period: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct UsaspendingStateProfile {
    code: String,
    fips: String,
    total_prime_amount: Option<f64>,
    total_prime_awards: Option<i64>,
}

#[derive(Debug, PartialEq)]
struct DolMinimumWageRecord {
    state_code: String,
    state_rate: Option<f64>,
    effective_rate: f64,
    category: String,
    has_footnote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FhfaStateHpiObservation {
    year: i32,
    quarter: u32,
    index: f64,
}

#[derive(Debug, PartialEq)]
struct CensusAcsStateMetrics {
    state_code: String,
    median_household_income: Option<f64>,
    median_gross_rent: Option<f64>,
    rent_burden_rate: Option<f64>,
    poverty_rate: Option<f64>,
}

async fn import_bls_laus_state_unemployment(pool: &PgPool) -> Result<ImportSummary> {
    let current_year = chrono::Utc::now().year();
    let start_year = current_year - 5;
    let series: Vec<String> = state_fips()
        .iter()
        .map(|(_, fips)| format!("LAUST{fips}0000000000003"))
        .collect();
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let mut imported = 0;
    let mut errors = 0;

    for (chunk_index, chunk) in series.chunks(25).enumerate() {
        let body = serde_json::json!({
            "seriesid": chunk,
            "startyear": start_year.to_string(),
            "endyear": current_year.to_string(),
        });

        let response = client
            .post("https://api.bls.gov/publicAPI/v2/timeseries/data/")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("BLS LAUS returned status {}", response.status());
            errors += 1;
            continue;
        }

        let body_text = response.text().await?;
        let hash = content_hash(&body_text);
        let parsed: BlsResponse = serde_json::from_str(&body_text)?;

        if parsed.status.as_deref() != Some("REQUEST_SUCCEEDED") {
            warn!("BLS LAUS request did not succeed: {:?}", parsed.message);
            errors += 1;
            continue;
        }

        let source_record = queries::upsert_source_record(
            pool,
            "bls-laus",
            "timeseries",
            &format!("state-unemployment-{current_year}-chunk-{chunk_index}"),
            Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
            &hash,
            None,
        )
        .await?;
        let evidence = queries::create_evidence(
            pool,
            source_record.id,
            Some("$.Results.series"),
            Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
            Some(1.0),
        )
        .await?;

        for series in parsed
            .results
            .map(|results| results.series)
            .unwrap_or_default()
        {
            if import_bls_laus_series(pool, &series, evidence.id).await? {
                imported += 1;
            }
        }
    }

    Ok(ImportSummary {
        source: "bls-laus".into(),
        records_imported: imported,
        records_skipped: 0,
        entities_created: 0,
        errors,
    })
}

async fn import_bls_laus_series(
    pool: &PgPool,
    series: &BlsSeries,
    evidence_id: uuid::Uuid,
) -> Result<bool> {
    let Some((state_code, _)) = state_fips()
        .iter()
        .find(|(_, fips)| series.series_id == format!("LAUST{fips}0000000000003"))
    else {
        return Ok(false);
    };

    let mut stored = 0;
    for observation in series
        .data
        .iter()
        .filter(|item| item.period.starts_with('M') && item.period != "M13")
    {
        let Ok(year) = observation.year.parse::<i32>() else {
            continue;
        };
        let Ok(month) = observation.period.trim_start_matches('M').parse::<u32>() else {
            continue;
        };
        let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) else {
            continue;
        };
        let Ok(value) = observation.value.parse::<f64>() else {
            continue;
        };

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "unemployment_rate".into(),
                geo_id: (*state_code).into(),
                date,
                value,
                vintage_date: chrono::Utc::now().date_naive(),
                release_date: None,
                source_series_id: Some(series.series_id.clone()),
                evidence_id: Some(evidence_id),
                notes: Some("Live BLS LAUS state unemployment rate.".into()),
            },
        )
        .await?;
        stored += 1;
    }

    Ok(stored > 0)
}

async fn import_bls_cpi_and_average_prices(pool: &PgPool) -> Result<ImportSummary> {
    let current_year = chrono::Utc::now().year();
    let start_year = current_year - 5;
    let series_ids: Vec<&str> = bls_cpi_price_series()
        .iter()
        .map(|definition| definition.series_id)
        .collect();
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let body = serde_json::json!({
        "seriesid": series_ids,
        "startyear": start_year.to_string(),
        "endyear": current_year.to_string(),
    });
    let response = client
        .post("https://api.bls.gov/publicAPI/v2/timeseries/data/")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        warn!("BLS CPI/prices returned status {}", response.status());
        return Ok(ImportSummary {
            source: "bls-cpi-prices".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let parsed: BlsResponse = serde_json::from_str(&body_text)?;

    if parsed.status.as_deref() != Some("REQUEST_SUCCEEDED") {
        warn!(
            "BLS CPI/prices request did not succeed: {:?}",
            parsed.message
        );
        return Ok(ImportSummary {
            source: "bls-cpi-prices".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let source_record = queries::upsert_source_record(
        pool,
        "bls-cpi-prices",
        "timeseries",
        &format!("us-cpi-prices-{}-{}", current_year - 1, current_year),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("$.Results.series"),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        Some(1.0),
    )
    .await?;

    let mut imported = 0;
    let mut skipped = 0;

    for series in parsed
        .results
        .map(|results| results.series)
        .unwrap_or_default()
    {
        let Some(definition) = bls_cpi_price_series()
            .iter()
            .find(|definition| definition.series_id == series.series_id)
        else {
            skipped += 1;
            continue;
        };

        for observation in all_monthly_bls_observations(&series.data) {
            let Some(date) = bls_observation_date(observation) else {
                skipped += 1;
                continue;
            };
            let Ok(level_value) = observation.value.parse::<f64>() else {
                skipped += 1;
                continue;
            };
            if level_value <= 0.0 {
                skipped += 1;
                continue;
            }

            let stored_value = if definition.value_kind == BlsCpiValueKind::YearOverYearPercent {
                let Some(prior) = same_month_prior_year_bls_observation(&series.data, observation)
                else {
                    skipped += 1;
                    continue;
                };
                let Ok(prior_value) = prior.value.parse::<f64>() else {
                    skipped += 1;
                    continue;
                };
                if prior_value <= 0.0 {
                    skipped += 1;
                    continue;
                }
                (level_value / prior_value - 1.0) * 100.0
            } else {
                level_value
            };

            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: definition.metric_id.into(),
                    geo_id: "US".into(),
                    date,
                    value: stored_value,
                    vintage_date: chrono::Utc::now().date_naive(),
                    release_date: None,
                    source_series_id: Some(series.series_id.clone()),
                    evidence_id: Some(evidence.id),
                    notes: Some(definition.notes.into()),
                },
            )
            .await?;
            imported += 1;
        }
    }

    Ok(ImportSummary {
        source: "bls-cpi-prices".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

async fn import_bls_ces_national_metrics(pool: &PgPool) -> Result<ImportSummary> {
    let current_year = chrono::Utc::now().year();
    let start_year = current_year - 5;
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let body = serde_json::json!({
        "seriesid": ["CES0000000001", "CES0500000003"],
        "startyear": start_year.to_string(),
        "endyear": current_year.to_string(),
    });
    let response = client
        .post("https://api.bls.gov/publicAPI/v2/timeseries/data/")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        warn!("BLS CES returned status {}", response.status());
        return Ok(ImportSummary {
            source: "bls-ces".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let parsed: BlsResponse = serde_json::from_str(&body_text)?;

    if parsed.status.as_deref() != Some("REQUEST_SUCCEEDED") {
        warn!("BLS CES request did not succeed: {:?}", parsed.message);
        return Ok(ImportSummary {
            source: "bls-ces".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let source_record = queries::upsert_source_record(
        pool,
        "bls-ces",
        "timeseries",
        &format!("national-ces-{}-{}", current_year - 1, current_year),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("$.Results.series"),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        Some(1.0),
    )
    .await?;

    let mut imported = 0;
    let mut skipped = 0;

    for series in parsed
        .results
        .map(|results| results.series)
        .unwrap_or_default()
    {
        for observation in all_monthly_bls_observations(&series.data) {
            let Some(date) = bls_observation_date(observation) else {
                skipped += 1;
                continue;
            };
            let Ok(value) = observation.value.parse::<f64>() else {
                skipped += 1;
                continue;
            };

            let (metric_id, stored_value, notes) = match series.series_id.as_str() {
                "CES0000000001" => (
                    "total_nonfarm_employment",
                    value * 1_000.0,
                    "Live BLS CES total nonfarm payroll employment; seasonally adjusted.",
                ),
                "CES0500000003" => (
                    "avg_hourly_earnings",
                    value,
                    "Live BLS CES average hourly earnings of all employees, total private; seasonally adjusted.",
                ),
                _ => {
                    skipped += 1;
                    continue;
                }
            };

            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: metric_id.into(),
                    geo_id: "US".into(),
                    date,
                    value: stored_value,
                    vintage_date: chrono::Utc::now().date_naive(),
                    release_date: None,
                    source_series_id: Some(series.series_id.clone()),
                    evidence_id: Some(evidence.id),
                    notes: Some(notes.into()),
                },
            )
            .await?;
            imported += 1;
        }

        let latest = latest_monthly_bls_observation(&series.data);
        if let Some(latest) = latest {
            let Ok(latest_value) = latest.value.parse::<f64>() else {
                continue;
            };
            if series.series_id == "CES0000000001"
                && let Some(previous) = previous_monthly_bls_observation(&series.data, latest)
                && let Ok(previous_value) = previous.value.parse::<f64>()
                && let Some(date) = bls_observation_date(latest)
            {
                let change = (latest_value - previous_value) * 1_000.0;
                queries::upsert_metric_observation(
                    pool,
                    &MetricObservation {
                        metric_id: "payroll_jobs_change".into(),
                        geo_id: "US".into(),
                        date,
                        value: change,
                        vintage_date: chrono::Utc::now().date_naive(),
                        release_date: None,
                        source_series_id: Some(series.series_id.clone()),
                        evidence_id: Some(evidence.id),
                        notes: Some("Live BLS CES monthly change in total nonfarm payroll employment; seasonally adjusted.".into()),
                    },
                )
                .await?;
                imported += 1;
            }
        }
    }

    Ok(ImportSummary {
        source: "bls-ces".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

async fn import_usaspending_state_totals(pool: &PgPool) -> Result<ImportSummary> {
    let client = reqwest::Client::builder().build()?;
    let today = chrono::Utc::now().date_naive();
    let mut imported = 0;
    let mut errors = 0;

    for (state_code, fips) in state_fips() {
        let source_url =
            format!("https://api.usaspending.gov/api/v2/recipient/state/{fips}/?year=latest");
        let response = client.get(&source_url).send().await?;

        if !response.status().is_success() {
            warn!(
                "USAspending state profile returned status {} for {}",
                response.status(),
                state_code
            );
            errors += 1;
            continue;
        }

        let body_text = response.text().await?;
        let hash = content_hash(&body_text);
        let parsed: UsaspendingStateProfile = serde_json::from_str(&body_text)?;

        if parsed.code != *state_code || parsed.fips != *fips {
            warn!(
                "USAspending state profile code mismatch for {}: code={} fips={}",
                state_code, parsed.code, parsed.fips
            );
            errors += 1;
            continue;
        }

        let Some(total_prime_amount) = parsed.total_prime_amount else {
            continue;
        };

        let source_record = queries::upsert_source_record(
            pool,
            "usaspending-state",
            "state-profile",
            &format!("{state_code}-latest"),
            Some(&source_url),
            &hash,
            None,
        )
        .await?;
        let evidence = queries::create_evidence(
            pool,
            source_record.id,
            Some("$.total_prime_amount"),
            Some(&source_url),
            Some(1.0),
        )
        .await?;

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "federal_contract_obligations".into(),
                geo_id: (*state_code).into(),
                date: today,
                value: total_prime_amount,
                vintage_date: today,
                release_date: None,
                source_series_id: Some(format!("recipient/state/{fips}?year=latest")),
                evidence_id: Some(evidence.id),
                notes: Some(format!(
                    "Live USAspending state profile total_prime_amount for latest trailing period; includes {} prime award records.",
                    parsed.total_prime_awards.unwrap_or(0)
                )),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(ImportSummary {
        source: "usaspending-state".into(),
        records_imported: imported,
        records_skipped: 0,
        entities_created: 0,
        errors,
    })
}

async fn import_dol_minimum_wages(pool: &PgPool) -> Result<ImportSummary> {
    let source_url = "https://www.dol.gov/agencies/whd/minimum-wage/state";
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let body_text = match client.get(source_url).send().await {
        Ok(response) if response.status().is_success() => response.text().await?,
        Ok(response) => {
            warn!(
                "DOL minimum wage page returned status {}; retrying with curl fallback",
                response.status()
            );
            fetch_with_curl(source_url).await?
        }
        Err(error) => {
            warn!("DOL minimum wage request failed: {error}; retrying with curl fallback");
            fetch_with_curl(source_url).await?
        }
    };
    let hash = content_hash(&body_text);
    let records = parse_dol_minimum_wage_records(&body_text)?;
    let source_record = queries::upsert_source_record(
        pool,
        "dol-min-wage",
        "state-page",
        "current",
        Some(source_url),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("embedded JavaScript vMinStateData"),
        Some(source_url),
        Some(0.9),
    )
    .await?;

    let today = chrono::Utc::now().date_naive();
    let mut imported = 0;
    let mut skipped = 0;

    for record in records {
        if !state_fips()
            .iter()
            .any(|(state_code, _)| *state_code == record.state_code)
        {
            skipped += 1;
            continue;
        }

        let state_minimum_wage = record.state_rate.unwrap_or(record.effective_rate);
        let state_minimum_note = if record.state_rate.is_some() {
            "Live DOL state basic minimum wage"
        } else {
            "DOL reports no state basic rate; federal minimum wage applies"
        };

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "state_min_wage".into(),
                geo_id: record.state_code.clone(),
                date: today,
                value: state_minimum_wage,
                vintage_date: today,
                release_date: None,
                source_series_id: Some(format!("dol-state-minimum-wage:{}", record.state_code)),
                evidence_id: Some(evidence.id),
                notes: Some(format!(
                    "{}. Category: {}. Footnote: {}.",
                    state_minimum_note,
                    record.category,
                    if record.has_footnote { "yes" } else { "no" }
                )),
            },
        )
        .await?;
        imported += 1;

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "effective_min_wage".into(),
                geo_id: record.state_code.clone(),
                date: today,
                value: record.effective_rate,
                vintage_date: today,
                release_date: None,
                source_series_id: Some(format!("dol-effective-minimum-wage:{}", record.state_code)),
                evidence_id: Some(evidence.id),
                notes: Some(format!(
                    "Effective floor used for affordability calculations: max(parsed state basic rate, federal minimum wage). Category: {}.",
                    record.category
                )),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(ImportSummary {
        source: "dol-min-wage".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

async fn import_fhfa_state_hpi_yoy(pool: &PgPool) -> Result<ImportSummary> {
    let source_url = "https://www.fhfa.gov/hpi/download/quarterly_datasets/hpi_at_state.txt";
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let response = client.get(source_url).send().await?;

    if !response.status().is_success() {
        warn!("FHFA HPI state file returned status {}", response.status());
        return Ok(ImportSummary {
            source: "fhfa-hpi".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let source_record = queries::upsert_source_record(
        pool,
        "fhfa-hpi",
        "quarterly-state-hpi",
        "hpi_at_state",
        Some(source_url),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("tab-delimited state quarterly HPI rows"),
        Some(source_url),
        Some(1.0),
    )
    .await?;

    let yoy_by_state = parse_fhfa_state_hpi_yoy(&body_text)?;
    let today = chrono::Utc::now().date_naive();
    let mut imported = 0;
    let mut skipped = 0;

    for (state_code, observation, yoy) in yoy_by_state {
        if !state_fips()
            .iter()
            .any(|(known_state, _)| *known_state == state_code)
        {
            skipped += 1;
            continue;
        }

        let Some(date) = quarter_start_date(observation.year, observation.quarter) else {
            skipped += 1;
            continue;
        };

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "fhfa_hpi_yoy".into(),
                geo_id: state_code.clone(),
                date,
                value: yoy,
                vintage_date: today,
                release_date: None,
                source_series_id: Some(format!("fhfa-hpi-at-state:{state_code}")),
                evidence_id: Some(evidence.id),
                notes: Some(format!(
                    "Live FHFA quarterly all-transactions state HPI YoY change for {}Q{}; not seasonally adjusted.",
                    observation.year, observation.quarter
                )),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(ImportSummary {
        source: "fhfa-hpi".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

async fn import_census_acs_state_metrics(pool: &PgPool) -> Result<ImportSummary> {
    let api_key = std::env::var("CENSUS_API_KEY")?;
    let source_url = format!(
        "https://api.census.gov/data/2024/acs/acs5?get=NAME,B19013_001E,B25064_001E,B25070_001E,B25070_007E,B25070_008E,B25070_009E,B25070_010E,B17001_001E,B17001_002E&for=state:*&key={api_key}"
    );
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let response = client.get(&source_url).send().await?;

    if !response.status().is_success() {
        warn!(
            "Census ACS state request returned status {}",
            response.status()
        );
        return Ok(ImportSummary {
            source: "census-acs".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let metrics = parse_census_acs_state_metrics(&body_text)?;
    let safe_source_url = source_url.replace(&api_key, "REDACTED");
    let source_record = queries::upsert_source_record(
        pool,
        "census-acs",
        "acs5-state",
        "2024-state-economic-housing",
        Some(&safe_source_url),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("ACS 2024 5-year state detailed table variables"),
        Some(&safe_source_url),
        Some(1.0),
    )
    .await?;

    let date = NaiveDate::from_ymd_opt(2024, 12, 31).expect("valid ACS date");
    let today = chrono::Utc::now().date_naive();
    let mut imported = 0;
    let mut skipped = 0;

    for row in metrics {
        for (metric_id, value, variable) in [
            (
                "median_household_income",
                row.median_household_income,
                "B19013_001E",
            ),
            ("median_gross_rent", row.median_gross_rent, "B25064_001E"),
            ("rent_burden_rate", row.rent_burden_rate, "B25070_007E-010E"),
            ("poverty_rate", row.poverty_rate, "B17001_002E/B17001_001E"),
        ] {
            let Some(value) = value else {
                skipped += 1;
                continue;
            };

            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: metric_id.into(),
                    geo_id: row.state_code.clone(),
                    date,
                    value,
                    vintage_date: today,
                    release_date: None,
                    source_series_id: Some(format!("acs5-2024:{variable}")),
                    evidence_id: Some(evidence.id),
                    notes: Some(
                        "Live Census ACS 2024 5-year state estimate; margins of error are not yet stored."
                            .into(),
                    ),
                },
            )
            .await?;
            imported += 1;
        }
    }

    let quintiles_url = format!(
        "https://api.census.gov/data/2024/acs/acs5?get=NAME,B19081_001E,B19081_002E,B19081_003E,B19081_004E,B19081_005E,B19081_006E&for=state:*&key={api_key}"
    );
    if let Ok(response) = client.get(&quintiles_url).send().await
        && response.status().is_success()
    {
        let body_text = response.text().await?;
        let rows: Vec<Vec<String>> = serde_json::from_str(&body_text).unwrap_or_default();
        if let Some(headers) = rows.first() {
            let idx = |n: &str| headers.iter().position(|h| h == n);
            if let (Some(st_idx), Some(v0), Some(v1), Some(v2), Some(v3), Some(v4), Some(_v5)) = (
                idx("state"),
                idx("B19081_001E"),
                idx("B19081_002E"),
                idx("B19081_003E"),
                idx("B19081_004E"),
                idx("B19081_005E"),
                idx("B19081_006E"),
            ) {
                for row in rows.iter().skip(1) {
                    let Some(fips) = row.get(st_idx) else {
                        continue;
                    };
                    let Some((state_code, _)) = state_fips().iter().find(|(_, f)| f == fips) else {
                        continue;
                    };
                    if let Some(v) = val_at(row, v0) {
                        queries::upsert_metric_observation(
                                pool,
                                &MetricObservation {
                                    metric_id: "quintile_income_top".into(),
                                    geo_id: (*state_code).into(),
                                    date,
                                    value: v * 5.0,
                                    vintage_date: today,
                                    release_date: None,
                                    source_series_id: Some(
                                        "acs5-2024:B19081_006E".into(),
                                    ),
                                    evidence_id: Some(evidence.id),
                                    notes: Some(
                                        "Derived top-quintile mean income = B19081_001E * 5 minus lower-quintile sums."
                                            .into(),
                                    ),
                                },
                            )
                            .await?;
                        imported += 1;
                    }
                    for (metric_id, col) in [
                        ("quintile_income_bottom", v1),
                        ("quintile_income_second", v2),
                        ("quintile_income_third", v3),
                        ("quintile_income_fourth", v4),
                    ] {
                        if let Some(v) = val_at(row, col) {
                            queries::upsert_metric_observation(
                                pool,
                                &MetricObservation {
                                    metric_id: metric_id.into(),
                                    geo_id: (*state_code).into(),
                                    date,
                                    value: v,
                                    vintage_date: today,
                                    release_date: None,
                                    source_series_id: Some(format!("acs5-2024:B19081_{col}")),
                                    evidence_id: Some(evidence.id),
                                    notes: Some(
                                        "Live Census ACS 2024 5-year state quintile mean income."
                                            .into(),
                                    ),
                                },
                            )
                            .await?;
                            imported += 1;
                        }
                    }
                }
            }
        }
    }

    let gender_url = format!(
        "https://api.census.gov/data/2024/acs/acs5?get=NAME,B20017_002E,B20017_003E&for=state:*&key={api_key}"
    );
    if let Ok(response) = client.get(&gender_url).send().await
        && response.status().is_success()
    {
        let body_text = response.text().await?;
        let rows: Vec<Vec<String>> = serde_json::from_str(&body_text).unwrap_or_default();
        if let Some(headers) = rows.first() {
            let st_idx = headers.iter().position(|h| h == "state");
            let male_idx = headers.iter().position(|h| h == "B20017_002E");
            let female_idx = headers.iter().position(|h| h == "B20017_003E");
            if let (Some(st_idx), Some(m_idx), Some(f_idx)) = (st_idx, male_idx, female_idx) {
                for row in rows.iter().skip(1) {
                    let Some(fips) = row.get(st_idx) else {
                        continue;
                    };
                    let Some((state_code, _)) = state_fips().iter().find(|(_, f)| f == fips) else {
                        continue;
                    };
                    for (metric_id, col) in [
                        ("median_earnings_male", m_idx),
                        ("median_earnings_female", f_idx),
                    ] {
                        if let Some(v) = val_at(row, col) {
                            queries::upsert_metric_observation(
                                pool,
                                &MetricObservation {
                                    metric_id: metric_id.into(),
                                    geo_id: (*state_code).into(),
                                    date,
                                    value: v,
                                    vintage_date: today,
                                    release_date: None,
                                    source_series_id: Some(format!("acs5-2024:B20017_{col}")),
                                    evidence_id: Some(evidence.id),
                                    notes: Some(
                                        "Live Census ACS 2024 5-year state median earnings by sex."
                                            .into(),
                                    ),
                                },
                            )
                            .await?;
                            imported += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(ImportSummary {
        source: "census-acs".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

fn parse_census_acs_state_metrics(body: &str) -> Result<Vec<CensusAcsStateMetrics>> {
    let rows: Vec<Vec<String>> = serde_json::from_str(body)?;
    let Some(headers) = rows.first() else {
        return Ok(Vec::new());
    };

    let index = |name: &str| headers.iter().position(|header| header == name);
    let Some(state_index) = index("state") else {
        return Ok(Vec::new());
    };

    let mut parsed_rows = Vec::new();
    for row in rows.iter().skip(1) {
        let Some(fips) = row.get(state_index) else {
            continue;
        };
        let Some((state_code, _)) = state_fips()
            .iter()
            .find(|(_, known_fips)| known_fips == fips)
        else {
            continue;
        };

        let value = |variable: &str| {
            index(variable)
                .and_then(|position| row.get(position))
                .and_then(|raw| parse_census_numeric(raw))
        };
        let rent_burden_total = value("B25070_001E");
        let rent_burden_count = ["B25070_007E", "B25070_008E", "B25070_009E", "B25070_010E"]
            .into_iter()
            .filter_map(value)
            .sum::<f64>();
        let poverty_total = value("B17001_001E");
        let poverty_count = value("B17001_002E");

        parsed_rows.push(CensusAcsStateMetrics {
            state_code: (*state_code).into(),
            median_household_income: value("B19013_001E"),
            median_gross_rent: value("B25064_001E"),
            rent_burden_rate: rent_burden_total
                .filter(|total| *total > 0.0)
                .map(|total| rent_burden_count / total * 100.0),
            poverty_rate: poverty_total
                .zip(poverty_count)
                .filter(|(total, _)| *total > 0.0)
                .map(|(total, count)| count / total * 100.0),
        });
    }

    Ok(parsed_rows)
}

fn parse_census_numeric(raw: &str) -> Option<f64> {
    let value = raw.parse::<f64>().ok()?;
    if value < 0.0 { None } else { Some(value) }
}

fn val_at(row: &[String], col: usize) -> Option<f64> {
    row.get(col).and_then(|v| parse_census_numeric(v))
}

async fn import_hud_fmr_state_rents(pool: &PgPool) -> Result<ImportSummary> {
    let api_key = std::env::var("HUD_FMR_API_KEY")?;
    let year = chrono::Utc::now().year();
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let auth_header = format!("Bearer {api_key}");

    let states_response = client
        .get("https://www.huduser.gov/hudapi/public/fmr/listStates")
        .header("Authorization", &auth_header)
        .send()
        .await?;

    if !states_response.status().is_success() {
        anyhow::bail!("HUD listStates returned {}", states_response.status());
    }

    let state_list: Vec<String> = states_response
        .json::<serde_json::Value>()
        .await?
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    s.get("state_code")
                        .and_then(|c| c.as_str().map(String::from))
                })
                .collect()
        })
        .unwrap_or_default();

    let known_states: Vec<&str> = state_fips().iter().map(|(code, _)| *code).collect();

    let date = NaiveDate::from_ymd_opt(year, 10, 1).expect("valid FMR effective date");
    let today = chrono::Utc::now().date_naive();
    let mut imported = 0_u64;
    let mut skipped = 0_u64;

    for state_code in &state_list {
        if !known_states.contains(&state_code.as_str()) {
            skipped += 1;
            continue;
        }

        let url =
            format!("https://www.huduser.gov/hudapi/public/fmr/statedata/{state_code}?year={year}");
        let resp = client
            .get(&url)
            .header("Authorization", &auth_header)
            .send()
            .await?;

        if !resp.status().is_success() {
            warn!("HUD statedata for {state_code} returned {}", resp.status());
            skipped += 1;
            continue;
        }

        let body = resp.text().await?;
        let median = parse_hud_fmr_statedata_median(&body).unwrap_or(None);
        let Some(fmr) = median else {
            warn!("HUD statedata for {state_code}: no county data");
            skipped += 1;
            continue;
        };

        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: "fmr_2br".into(),
                geo_id: state_code.clone(),
                date,
                value: fmr,
                vintage_date: today,
                release_date: None,
                source_series_id: Some(format!("hud-fmr-{year}:{state_code}")),
                evidence_id: None,
                notes: Some(format!(
                    "Live HUD FY{year} state-level 2-bedroom fair market rent median across counties."
                )),
            },
        )
        .await?;
        imported += 1;
    }

    if imported == 0 {
        anyhow::bail!("HUD FMR: no state data could be imported");
    }

    Ok(ImportSummary {
        source: "hud-fmr".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

fn parse_hud_fmr_statedata_median(body: &str) -> Result<Option<f64>> {
    let parsed: serde_json::Value = serde_json::from_str(body)?;
    let counties = parsed
        .get("data")
        .and_then(|d| d.get("counties"))
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let mut values: Vec<f64> = Vec::new();
    for county in &counties {
        if let Some(fmr) = county.get("Two-Bedroom").and_then(|v| v.as_f64()) {
            values.push(fmr);
        }
    }

    if values.is_empty() {
        return Ok(None);
    }

    values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let len = values.len();
    let median = if len.is_multiple_of(2) {
        (values[len / 2 - 1] + values[len / 2]) / 2.0
    } else {
        values[len / 2]
    };

    Ok(Some(median))
}

#[cfg(test)]
fn parse_hud_fmr_state_medians(body: &str) -> Result<Vec<(String, f64)>> {
    let parsed: serde_json::Value = serde_json::from_str(body)?;
    let data = parsed
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| anyhow::anyhow!("missing data array"))?;

    let mut state_groups: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();

    for item in data {
        if let Some(state) = item.get("state_alpha").and_then(|s| s.as_str()) {
            if state.is_empty() {
                continue;
            }
            let fmr_val = if let Some(val) = item.get("fmr_2") {
                if let Some(num) = val.as_f64() {
                    Some(num)
                } else if let Some(s) = val.as_str() {
                    s.parse::<f64>().ok()
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(val) = fmr_val {
                state_groups.entry(state.to_string()).or_default().push(val);
            }
        }
    }

    let mut results = Vec::new();
    for (state, mut values) in state_groups {
        if values.is_empty() {
            continue;
        }
        values.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = values.len();
        let median = if len.is_multiple_of(2) {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        };
        results.push((state, median));
    }

    results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

async fn import_eia_gas_prices(pool: &PgPool) -> Result<ImportSummary> {
    let api_key = std::env::var("EIA_API_KEY")?;
    let url = format!(
        "https://api.eia.gov/v2/petroleum/pri/gnd/data/?frequency=weekly&data[]=value&facets[series][]=EMD_EPD2D_PTE_R10_DPG&sort[0][column]=period&sort[0][direction]=desc&length=1&api_key={api_key}"
    );
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        warn!("EIA gas API returned status {}", response.status());
        return Ok(ImportSummary {
            source: "eia-gas".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let safe_url = "https://api.eia.gov/v2/petroleum/pri/gnd/data/?key=REDACTED".to_string();
    let source_record = queries::upsert_source_record(
        pool,
        "eia-gas",
        "petroleum-prices",
        "regular-gas-price",
        Some(&safe_url),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("$.response.data"),
        Some(&safe_url),
        Some(1.0),
    )
    .await?;

    let price = parse_eia_latest_gas_price(&body_text)?;
    let Some(price) = price else {
        return Ok(ImportSummary {
            source: "eia-gas".into(),
            records_imported: 0,
            records_skipped: 1,
            entities_created: 0,
            errors: 0,
        });
    };

    let today = chrono::Utc::now().date_naive();
    queries::upsert_metric_observation(
        pool,
        &MetricObservation {
            metric_id: "regular_gas_price".into(),
            geo_id: "US".into(),
            date: today,
            value: price,
            vintage_date: today,
            release_date: None,
            source_series_id: Some("EMD_EPD2D_PTE_R10_DPG".into()),
            evidence_id: Some(evidence.id),
            notes: Some("Live EIA U.S. regular conventional retail gasoline price, weekly.".into()),
        },
    )
    .await?;

    Ok(ImportSummary {
        source: "eia-gas".into(),
        records_imported: 1,
        records_skipped: 0,
        entities_created: 0,
        errors: 0,
    })
}

fn parse_eia_latest_gas_price(body: &str) -> Result<Option<f64>> {
    let parsed: serde_json::Value = serde_json::from_str(body)?;
    let records = parsed
        .get("response")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    for record in &records {
        let value = record.get("value").and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        });
        if let Some(value) = value
            && value > 0.0
        {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

async fn import_bea_regional_economic_accounts(pool: &PgPool) -> Result<ImportSummary> {
    let api_key = std::env::var("BEA_API_KEY")?;
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let year = chrono::Utc::now().year() - 1;

    let mut imported = 0;
    let mut errors = 0;
    let mut skipped = 0;

    for (dataset, line_code, metric_id) in [
        ("SQGDP9", "1", "state_gdp"),
        ("SAINC1", "1", "personal_income"),
    ] {
        let url = format!(
            "https://apps.bea.gov/api/data/?UserID={api_key}&method=GetData&datasetname=Regional&TableName={dataset}&GeoFips=STATE&LineCode={line_code}&Year={year}&ResultFormat=JSON"
        );
        let safe_url = format!(
            "https://apps.bea.gov/api/data/?UserID=REDACTED&method=GetData&datasetname=Regional&TableName={dataset}&GeoFips=STATE&LineCode={line_code}&Year={year}&ResultFormat=JSON"
        );
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            warn!("BEA {} API returned status {}", dataset, response.status());
            errors += 1;
            continue;
        }

        let body_text = response.text().await?;
        let hash = content_hash(&body_text);
        let source_record = queries::upsert_source_record(
            pool,
            "bea-regional",
            dataset,
            &format!("{dataset}-{year}"),
            Some(&safe_url),
            &hash,
            None,
        )
        .await?;
        let evidence = queries::create_evidence(
            pool,
            source_record.id,
            Some("$.BEAAPI.Results.Data"),
            Some(&safe_url),
            Some(1.0),
        )
        .await?;

        let parsed = parse_bea_geofips_responses(&body_text, metric_id)?;
        let date = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid BEA date");
        let today = chrono::Utc::now().date_naive();

        for (state_code, value) in parsed {
            if !state_fips().iter().any(|(known, _)| *known == state_code) {
                skipped += 1;
                continue;
            }

            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: metric_id.into(),
                    geo_id: state_code.clone(),
                    date,
                    value,
                    vintage_date: today,
                    release_date: None,
                    source_series_id: Some(format!("bea-{dataset}-{line_code}:{state_code}")),
                    evidence_id: Some(evidence.id),
                    notes: Some(format!(
                        "Live BEA {dataset} {line_code} for {year}; millions of current dollars."
                    )),
                },
            )
            .await?;
            imported += 1;
        }
    }

    Ok(ImportSummary {
        source: "bea-regional".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors,
    })
}

fn parse_bea_geofips_responses(body: &str, metric_id: &str) -> Result<Vec<(String, f64)>> {
    let parsed: serde_json::Value = serde_json::from_str(body)?;
    let data_rows = parsed
        .get("BEAAPI")
        .and_then(|bea| bea.get("Results"))
        .and_then(|results| results.get("Data"))
        .and_then(|data| data.as_array())
        .map(|arr| arr.to_vec())
        .unwrap_or_default();

    let mut rows: Vec<(String, f64)> = Vec::new();
    for row in data_rows {
        let geo_fips = row
            .get("GeoFips")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let value = row.get("DataValue").and_then(|v| v.as_str()).and_then(|s| {
            s.replace(',', "")
                .parse::<f64>()
                .ok()
                .map(|v| v * 1_000_000.0)
        });

        let Some((state_code, _)) = geo_fips.as_deref().and_then(|fips| {
            state_fips()
                .iter()
                .find(|(code, f)| fips.starts_with(f) || fips == *code)
        }) else {
            continue;
        };
        let Some(value) = value else {
            continue;
        };
        if value <= 0.0 {
            continue;
        }
        rows.push((state_code.to_string(), value));
    }

    if rows.is_empty() {
        anyhow::bail!("BEA {} response contained no usable state data", metric_id);
    }

    rows.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(rows)
}

async fn import_bls_cps_median_earnings(pool: &PgPool) -> Result<ImportSummary> {
    let current_year = chrono::Utc::now().year();
    let start_year = current_year - 5;
    let cps_series = bls_cps_earnings_series();
    let series_ids: Vec<&str> = cps_series.iter().map(|def| def.series_id).collect();
    let client = reqwest::Client::builder()
        .user_agent("FundingFlow/0.1 (public-record research)")
        .build()?;
    let body = serde_json::json!({
        "seriesid": series_ids,
        "startyear": start_year.to_string(),
        "endyear": current_year.to_string(),
    });
    let response = client
        .post("https://api.bls.gov/publicAPI/v2/timeseries/data/")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        warn!("BLS CPS returned status {}", response.status());
        return Ok(ImportSummary {
            source: "bls-cps".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let body_text = response.text().await?;
    let hash = content_hash(&body_text);
    let parsed: BlsResponse = serde_json::from_str(&body_text)?;

    if parsed.status.as_deref() != Some("REQUEST_SUCCEEDED") {
        warn!("BLS CPS request did not succeed: {:?}", parsed.message);
        return Ok(ImportSummary {
            source: "bls-cps".into(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 1,
        });
    }

    let source_record = queries::upsert_source_record(
        pool,
        "bls-cps",
        "timeseries",
        &format!("cps-median-earnings-{}-{}", start_year, current_year),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        &hash,
        None,
    )
    .await?;
    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        Some("$.Results.series"),
        Some("https://api.bls.gov/publicAPI/v2/timeseries/data/"),
        Some(1.0),
    )
    .await?;

    let mut imported = 0;
    let mut skipped = 0;

    for series in parsed
        .results
        .map(|results| results.series)
        .unwrap_or_default()
    {
        let Some(def) = cps_series
            .iter()
            .find(|def| def.series_id == series.series_id)
        else {
            skipped += 1;
            continue;
        };

        for observation in all_quarterly_bls_observations(&series.data) {
            let Some(date) = bls_observation_date(observation) else {
                continue;
            };
            let Ok(value) = observation.value.parse::<f64>() else {
                continue;
            };
            if value <= 0.0 {
                continue;
            }

            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: def.metric_id.into(),
                    geo_id: "US".into(),
                    date,
                    value,
                    vintage_date: chrono::Utc::now().date_naive(),
                    release_date: None,
                    source_series_id: Some(series.series_id.clone()),
                    evidence_id: Some(evidence.id),
                    notes: Some(def.notes.into()),
                },
            )
            .await?;
            imported += 1;
        }
    }

    Ok(ImportSummary {
        source: "bls-cps".into(),
        records_imported: imported,
        records_skipped: skipped,
        entities_created: 0,
        errors: 0,
    })
}

#[derive(Debug, Clone, Copy)]
struct BlsCpsEarningsSeries {
    series_id: &'static str,
    metric_id: &'static str,
    notes: &'static str,
}

fn bls_cps_earnings_series() -> &'static [BlsCpsEarningsSeries] {
    &[
        BlsCpsEarningsSeries {
            series_id: "LEU0252881500",
            metric_id: "median_weekly_earnings",
            notes: "Live BLS CPS median usual weekly earnings, full-time wage and salary workers.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252881800",
            metric_id: "median_weekly_earnings_male",
            notes: "Live BLS CPS median usual weekly earnings, men.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252882100",
            metric_id: "median_weekly_earnings_female",
            notes: "Live BLS CPS median usual weekly earnings, women.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252882700",
            metric_id: "median_weekly_earnings_white",
            notes: "Live BLS CPS median usual weekly earnings, White.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252883000",
            metric_id: "median_weekly_earnings_black",
            notes: "Live BLS CPS median usual weekly earnings, Black or African American.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252883300",
            metric_id: "median_weekly_earnings_asian",
            notes: "Live BLS CPS median usual weekly earnings, Asian.",
        },
        BlsCpsEarningsSeries {
            series_id: "LEU0252883600",
            metric_id: "median_weekly_earnings_hispanic",
            notes: "Live BLS CPS median usual weekly earnings, Hispanic or Latino.",
        },
    ]
}

fn parse_fhfa_state_hpi_yoy(body: &str) -> Result<Vec<(String, FhfaStateHpiObservation, f64)>> {
    let mut by_state: HashMap<String, Vec<FhfaStateHpiObservation>> = HashMap::new();

    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 4 {
            continue;
        }

        let state_code = fields[0].to_string();
        let observation = FhfaStateHpiObservation {
            year: fields[1].parse()?,
            quarter: fields[2].parse()?,
            index: fields[3].parse()?,
        };
        by_state.entry(state_code).or_default().push(observation);
    }

    let mut rows = Vec::new();
    for (state_code, mut observations) in by_state {
        observations.sort_by_key(|observation| (observation.year, observation.quarter));
        let Some(latest) = observations.last().copied() else {
            continue;
        };
        let Some(prior_year) = observations
            .iter()
            .find(|observation| {
                observation.year == latest.year - 1 && observation.quarter == latest.quarter
            })
            .copied()
        else {
            continue;
        };

        if prior_year.index <= 0.0 {
            continue;
        }

        let yoy = (latest.index / prior_year.index - 1.0) * 100.0;
        rows.push((state_code, latest, yoy));
    }

    rows.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(rows)
}

fn quarter_start_date(year: i32, quarter: u32) -> Option<NaiveDate> {
    let month = match quarter {
        1 => 1,
        2 => 4,
        3 => 7,
        4 => 10,
        _ => return None,
    };

    NaiveDate::from_ymd_opt(year, month, 1)
}

fn parse_dol_minimum_wage_records(html: &str) -> Result<Vec<DolMinimumWageRecord>> {
    let row_pattern = Regex::new(
        r#"\{\s*State:\s*"(?P<state>[^"]+)",\s*Wage:\s*"(?P<wage>[^"]*)",\s*Type:\s*"(?P<kind>[^"]+)",\s*FootNote:\s*"(?P<footnote>[^"]*)"\s*\}"#,
    )?;
    let records = row_pattern
        .captures_iter(html)
        .map(|captures| {
            let state_code = normalize_dol_state_code(&captures["state"]);
            let state_rate = extract_first_dollar_amount(&captures["wage"]);
            let effective_rate = state_rate.unwrap_or(0.0).max(7.25);

            DolMinimumWageRecord {
                state_code,
                state_rate,
                effective_rate,
                category: captures["kind"].to_string(),
                has_footnote: captures["footnote"].trim() == "Y",
            }
        })
        .collect();

    Ok(records)
}

async fn fetch_with_curl(url: &str) -> Result<String> {
    let output = Command::new("curl")
        .args(["-L", "-sS", url])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!(
            "curl fallback failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn normalize_dol_state_code(code: &str) -> String {
    match code {
        "WDC" => "DC",
        other => other,
    }
    .to_string()
}

fn extract_first_dollar_amount(value: &str) -> Option<f64> {
    let amount_pattern = Regex::new(r#"\$([0-9]+(?:\.[0-9]+)?)"#).ok()?;
    amount_pattern
        .captures(value)
        .and_then(|captures| captures.get(1))
        .and_then(|amount| amount.as_str().parse::<f64>().ok())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlsCpiValueKind {
    YearOverYearPercent,
    LatestLevel,
}

#[derive(Debug, Clone, Copy)]
struct BlsCpiPriceSeriesDefinition {
    series_id: &'static str,
    metric_id: &'static str,
    value_kind: BlsCpiValueKind,
    notes: &'static str,
}

fn bls_cpi_price_series() -> &'static [BlsCpiPriceSeriesDefinition] {
    &[
        BlsCpiPriceSeriesDefinition {
            series_id: "CUUR0000SAF11",
            metric_id: "food_at_home_cpi_yoy",
            value_kind: BlsCpiValueKind::YearOverYearPercent,
            notes: "Live BLS CPI-U U.S. city average food at home 12-month change; not seasonally adjusted.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "CUUR0000SEFV",
            metric_id: "food_away_from_home_cpi_yoy",
            value_kind: BlsCpiValueKind::YearOverYearPercent,
            notes: "Live BLS CPI-U U.S. city average food away from home 12-month change; not seasonally adjusted.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000708111",
            metric_id: "egg_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for eggs, grade A, large, per dozen; U.S. city average.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000709112",
            metric_id: "milk_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for whole fortified milk, per gallon; U.S. city average.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000702111",
            metric_id: "white_bread_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for white pan bread, per pound; U.S. city average.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000703112",
            metric_id: "ground_beef_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for 100 percent ground beef, per pound; U.S. city average.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000FF1101",
            metric_id: "chicken_breast_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for boneless chicken breast, per pound; U.S. city average.",
        },
        BlsCpiPriceSeriesDefinition {
            series_id: "APU0000711211",
            metric_id: "banana_price",
            value_kind: BlsCpiValueKind::LatestLevel,
            notes: "Live BLS average price for bananas, per pound; U.S. city average.",
        },
    ]
}

fn latest_monthly_bls_observation(data: &[BlsObservation]) -> Option<&BlsObservation> {
    data.iter()
        .filter(|item| item.period.starts_with('M') && item.period != "M13")
        .max_by_key(|item| (&item.year, &item.period))
}

fn all_monthly_bls_observations(data: &[BlsObservation]) -> Vec<&BlsObservation> {
    let mut observations: Vec<&BlsObservation> = data
        .iter()
        .filter(|item| item.period.starts_with('M') && item.period != "M13")
        .collect();
    observations.sort_by_key(|item| (&item.year, &item.period));
    observations
}

fn same_month_prior_year_bls_observation<'a>(
    data: &'a [BlsObservation],
    latest: &BlsObservation,
) -> Option<&'a BlsObservation> {
    let year = latest.year.parse::<i32>().ok()?;
    data.iter()
        .find(|item| item.year == (year - 1).to_string() && item.period == latest.period)
}

fn previous_monthly_bls_observation<'a>(
    data: &'a [BlsObservation],
    latest: &BlsObservation,
) -> Option<&'a BlsObservation> {
    let latest_date = bls_observation_date(latest)?;
    data.iter()
        .filter(|item| item.period.starts_with('M') && item.period != "M13")
        .filter(|item| {
            bls_observation_date(item)
                .map(|date| date < latest_date)
                .unwrap_or(false)
        })
        .max_by_key(|item| (&item.year, &item.period))
}

fn bls_observation_date(observation: &BlsObservation) -> Option<NaiveDate> {
    let year = observation.year.parse::<i32>().ok()?;
    if observation.period.starts_with('Q') {
        let quarter = observation
            .period
            .trim_start_matches('Q')
            .parse::<u32>()
            .ok()?;
        let month = match quarter {
            1 => 1,
            2 => 4,
            3 => 7,
            4 => 10,
            _ => return None,
        };
        NaiveDate::from_ymd_opt(year, month, 1)
    } else {
        let month = observation
            .period
            .trim_start_matches('M')
            .parse::<u32>()
            .ok()?;
        NaiveDate::from_ymd_opt(year, month, 1)
    }
}

fn all_quarterly_bls_observations(data: &[BlsObservation]) -> Vec<&BlsObservation> {
    let mut observations: Vec<&BlsObservation> = data
        .iter()
        .filter(|item| item.period.starts_with('Q'))
        .filter(|item| item.value != "-")
        .collect();
    observations.sort_by_key(|item| (&item.year, &item.period));
    observations
}

fn required_key_missing(source: EconomicSource) -> bool {
    let env_name = match source {
        EconomicSource::BeaRegional => "BEA_API_KEY",
        EconomicSource::CensusAcs => "CENSUS_API_KEY",
        EconomicSource::HudFmr => "HUD_FMR_API_KEY",
        EconomicSource::EiaGas => "EIA_API_KEY",
        _ => return false,
    };

    std::env::var(env_name)
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
}

pub async fn ensure_base_economic_catalog(pool: &PgPool) -> Result<()> {
    for geo in base_geos() {
        queries::upsert_geo(pool, &geo).await?;
    }

    for source in [
        EconomicSource::BlsLaus,
        EconomicSource::BlsCes,
        EconomicSource::BlsCpiPrices,
        EconomicSource::BlsCps,
        EconomicSource::DolMinWage,
        EconomicSource::BeaRegional,
        EconomicSource::CensusAcs,
        EconomicSource::FhfaHpi,
        EconomicSource::HudFmr,
        EconomicSource::EiaGas,
        EconomicSource::UsaspendingState,
    ] {
        queries::upsert_data_source(pool, &source.data_source()).await?;
        for metric in source.metrics() {
            queries::upsert_metric(pool, &metric).await?;
        }
    }

    Ok(())
}

pub async fn seed_fixture_state_metrics(pool: &PgPool) -> Result<ImportSummary> {
    ensure_base_economic_catalog(pool).await?;
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid fixture date");
    let vintage_date = NaiveDate::from_ymd_opt(2026, 6, 22).expect("valid fixture date");
    let release_date = Some(vintage_date);
    let mut imported = 0;

    for (metric_id, value) in [
        ("unemployment_rate", 4.3),
        ("payroll_jobs_change", 172_000.0),
        ("avg_hourly_earnings", 36.24),
        ("food_at_home_cpi_yoy", 2.7),
        ("food_away_from_home_cpi_yoy", 3.5),
        ("regular_gas_price", 3.18),
        ("median_weekly_earnings", 1_159.0),
        ("median_weekly_earnings_male", 1_261.0),
        ("median_weekly_earnings_female", 1_043.0),
        ("median_weekly_earnings_white", 1_177.0),
        ("median_weekly_earnings_black", 959.0),
        ("median_weekly_earnings_asian", 1_525.0),
        ("median_weekly_earnings_hispanic", 902.0),
    ] {
        queries::upsert_metric_observation(
            pool,
            &MetricObservation {
                metric_id: metric_id.into(),
                geo_id: "US".into(),
                date,
                value,
                vintage_date,
                release_date,
                source_series_id: None,
                evidence_id: None,
                notes: Some(
                    "Development fixture value for national pulse; replace with live agency import before public use."
                        .into(),
                ),
            },
        )
        .await?;
        imported += 1;
    }

    for (idx, geo) in base_geos()
        .into_iter()
        .filter(|geo| geo.geo_type == "state")
        .enumerate()
    {
        let offset = idx as f64;
        let observations = [
            ("unemployment_rate", 3.2 + (offset % 18.0) / 10.0),
            ("avg_hourly_earnings", 24.0 + (offset % 12.0)),
            (
                "state_min_wage",
                if geo.geo_id == "PA" {
                    7.25
                } else {
                    10.0 + (offset % 7.0)
                },
            ),
            ("median_gross_rent", 850.0 + offset * 18.0),
            ("fmr_2br", 1050.0 + offset * 21.0),
            ("rent_burden_rate", 24.0 + (offset % 12.0)),
            ("regular_gas_price", 3.0 + (offset % 8.0) / 20.0),
            ("state_gdp", 50_000_000_000.0 + offset * 25_000_000_000.0),
            (
                "federal_contract_obligations",
                500_000_000.0 + offset * 30_000_000.0,
            ),
            (
                "effective_min_wage",
                if geo.geo_id == "PA" {
                    7.25
                } else {
                    10.0 + (offset % 7.0)
                },
            ),
            ("fhfa_hpi_yoy", 3.0 + (offset % 10.0) / 10.0),
            ("poverty_rate", 12.0 + (offset % 14.0) / 2.0),
            ("median_household_income", 55_000.0 + offset * 1500.0),
            ("quintile_income_bottom", 14_000.0 + offset * 400.0),
            ("quintile_income_second", 35_000.0 + offset * 800.0),
            ("quintile_income_third", 55_000.0 + offset * 1200.0),
            ("quintile_income_fourth", 85_000.0 + offset * 2000.0),
            ("quintile_income_top", 145_000.0 + offset * 3000.0),
            ("median_earnings_male", 42_000.0 + offset * 1000.0),
            ("median_earnings_female", 35_000.0 + offset * 800.0),
        ];

        for (metric_id, value) in observations {
            queries::upsert_metric_observation(
                pool,
                &MetricObservation {
                    metric_id: metric_id.into(),
                    geo_id: geo.geo_id.clone(),
                    date,
                    value,
                    vintage_date,
                    release_date,
                    source_series_id: None,
                    evidence_id: None,
                    notes: Some("Development fixture value; replace with live agency import before public use.".into()),
                },
            )
            .await?;
            imported += 1;
        }
    }

    Ok(ImportSummary {
        source: "economic-fixture".into(),
        records_imported: imported,
        records_skipped: 0,
        entities_created: 0,
        errors: 0,
    })
}

pub async fn derive_state_mvp_metrics(pool: &PgPool) -> Result<ImportSummary> {
    let mut imported = 0;
    let vintage_date = NaiveDate::from_ymd_opt(2026, 6, 22).expect("valid fixture date");

    imported += derive_national_food_basket_metrics(pool, vintage_date).await?;

    for geo in base_geos()
        .into_iter()
        .filter(|geo| geo.geo_type == "state")
    {
        let metrics = queries::get_latest_metric_observations(pool, &geo.geo_id, None, 100).await?;
        let value = |metric_id: &str| {
            metrics
                .iter()
                .find(|item| item.observation.metric_id == metric_id)
                .map(|item| item.observation.value)
        };

        if let (Some(fmr), Some(min_wage)) = (
            value("fmr_2br"),
            value("effective_min_wage").or_else(|| value("state_min_wage")),
        ) && min_wage > 0.0
        {
            queries::upsert_derived_metric_observation(
                pool,
                &DerivedMetricObservation {
                    metric_id: "rent_hours_min_wage".into(),
                    geo_id: geo.geo_id.clone(),
                    date: vintage_date,
                    value: fmr / min_wage,
                    formula: "monthly_2br_fmr / effective_min_wage".into(),
                    input_metric_ids: vec!["fmr_2br".into(), "effective_min_wage".into()],
                    vintage_date,
                    notes: Some("Uses latest available observations by metric.".into()),
                },
            )
            .await?;
            imported += 1;
        }

        if let (Some(fmr), Some(avg_wage)) = (value("fmr_2br"), value("avg_hourly_earnings"))
            && avg_wage > 0.0
        {
            queries::upsert_derived_metric_observation(
                pool,
                &DerivedMetricObservation {
                    metric_id: "rent_hours_avg_wage".into(),
                    geo_id: geo.geo_id.clone(),
                    date: vintage_date,
                    value: fmr / avg_wage,
                    formula: "monthly_2br_fmr / avg_hourly_wage".into(),
                    input_metric_ids: vec!["fmr_2br".into(), "avg_hourly_earnings".into()],
                    vintage_date,
                    notes: Some("Uses latest available observations by metric.".into()),
                },
            )
            .await?;
            imported += 1;
        }

        if let (Some(obligations), Some(gdp)) =
            (value("federal_contract_obligations"), value("state_gdp"))
            && gdp > 0.0
        {
            queries::upsert_derived_metric_observation(
                pool,
                &DerivedMetricObservation {
                    metric_id: "contract_intensity".into(),
                    geo_id: geo.geo_id.clone(),
                    date: vintage_date,
                    value: obligations / gdp,
                    formula: "federal_contract_obligations / state_gdp".into(),
                    input_metric_ids: vec![
                        "federal_contract_obligations".into(),
                        "state_gdp".into(),
                    ],
                    vintage_date,
                    notes: Some("Uses latest available observations by metric.".into()),
                },
            )
            .await?;
            imported += 1;
        }
    }

    Ok(ImportSummary {
        source: "state-mvp-derived".into(),
        records_imported: imported,
        records_skipped: 0,
        entities_created: 0,
        errors: 0,
    })
}

async fn derive_national_food_basket_metrics(
    pool: &PgPool,
    vintage_date: NaiveDate,
) -> Result<u64> {
    let metrics = queries::get_latest_metric_observations(pool, "US", None, 100).await?;
    let value = |metric_id: &str| {
        metrics
            .iter()
            .find(|item| item.observation.metric_id == metric_id)
            .map(|item| item.observation.value)
    };

    let basket_inputs = [
        "egg_price",
        "milk_price",
        "white_bread_price",
        "ground_beef_price",
        "chicken_breast_price",
        "banana_price",
    ];
    let basket_values = basket_inputs
        .iter()
        .map(|metric_id| value(metric_id))
        .collect::<Option<Vec<f64>>>();
    let mut imported = 0;

    let Some(basket_values) = basket_values else {
        return Ok(imported);
    };

    let food_basket_cost = basket_values.iter().sum::<f64>();
    queries::upsert_derived_metric_observation(
        pool,
        &DerivedMetricObservation {
            metric_id: "food_basket_cost".into(),
            geo_id: "US".into(),
            date: vintage_date,
            value: food_basket_cost,
            formula: "egg_price + milk_price + white_bread_price + ground_beef_price + chicken_breast_price + banana_price".into(),
            input_metric_ids: basket_inputs.iter().map(|metric_id| (*metric_id).into()).collect(),
            vintage_date,
            notes: Some("One-unit BLS food basket: 1 dozen eggs, 1 gallon milk, 1 lb bread, 1 lb ground beef, 1 lb chicken breast, 1 lb bananas.".into()),
        },
    )
    .await?;
    imported += 1;

    if let Some(avg_hourly_earnings) = value("avg_hourly_earnings")
        && avg_hourly_earnings > 0.0
    {
        queries::upsert_derived_metric_observation(
            pool,
            &DerivedMetricObservation {
                metric_id: "food_basket_as_percent_weekly_wage".into(),
                geo_id: "US".into(),
                date: vintage_date,
                value: food_basket_cost / (avg_hourly_earnings * 40.0) * 100.0,
                formula: "food_basket_cost / (avg_hourly_earnings * 40) * 100".into(),
                input_metric_ids: vec!["food_basket_cost".into(), "avg_hourly_earnings".into()],
                vintage_date,
                notes: Some(
                    "Uses BLS U.S. city average food basket and national average hourly earnings."
                        .into(),
                ),
            },
        )
        .await?;
        imported += 1;
    }

    Ok(imported)
}

fn base_geos() -> Vec<Geo> {
    let mut geos = vec![Geo {
        geo_id: "US".into(),
        geo_type: "nation".into(),
        name: "United States".into(),
        state_code: None,
        county_code: None,
        region: None,
    }];

    geos.extend(
        [
            ("AL", "Alabama", "South"),
            ("AK", "Alaska", "West"),
            ("AZ", "Arizona", "West"),
            ("AR", "Arkansas", "South"),
            ("CA", "California", "West"),
            ("CO", "Colorado", "West"),
            ("CT", "Connecticut", "Northeast"),
            ("DE", "Delaware", "South"),
            ("FL", "Florida", "South"),
            ("GA", "Georgia", "South"),
            ("HI", "Hawaii", "West"),
            ("ID", "Idaho", "West"),
            ("IL", "Illinois", "Midwest"),
            ("IN", "Indiana", "Midwest"),
            ("IA", "Iowa", "Midwest"),
            ("KS", "Kansas", "Midwest"),
            ("KY", "Kentucky", "South"),
            ("LA", "Louisiana", "South"),
            ("ME", "Maine", "Northeast"),
            ("MD", "Maryland", "South"),
            ("MA", "Massachusetts", "Northeast"),
            ("MI", "Michigan", "Midwest"),
            ("MN", "Minnesota", "Midwest"),
            ("MS", "Mississippi", "South"),
            ("MO", "Missouri", "Midwest"),
            ("MT", "Montana", "West"),
            ("NE", "Nebraska", "Midwest"),
            ("NV", "Nevada", "West"),
            ("NH", "New Hampshire", "Northeast"),
            ("NJ", "New Jersey", "Northeast"),
            ("NM", "New Mexico", "West"),
            ("NY", "New York", "Northeast"),
            ("NC", "North Carolina", "South"),
            ("ND", "North Dakota", "Midwest"),
            ("OH", "Ohio", "Midwest"),
            ("OK", "Oklahoma", "South"),
            ("OR", "Oregon", "West"),
            ("PA", "Pennsylvania", "Northeast"),
            ("RI", "Rhode Island", "Northeast"),
            ("SC", "South Carolina", "South"),
            ("SD", "South Dakota", "Midwest"),
            ("TN", "Tennessee", "South"),
            ("TX", "Texas", "South"),
            ("UT", "Utah", "West"),
            ("VT", "Vermont", "Northeast"),
            ("VA", "Virginia", "South"),
            ("WA", "Washington", "West"),
            ("WV", "West Virginia", "South"),
            ("WI", "Wisconsin", "Midwest"),
            ("WY", "Wyoming", "West"),
            ("DC", "District of Columbia", "South"),
        ]
        .into_iter()
        .map(|(code, name, region)| Geo {
            geo_id: code.into(),
            geo_type: "state".into(),
            name: name.into(),
            state_code: Some(code.into()),
            county_code: None,
            region: Some(region.into()),
        }),
    );

    geos
}

fn state_fips() -> &'static [(&'static str, &'static str)] {
    &[
        ("AL", "01"),
        ("AK", "02"),
        ("AZ", "04"),
        ("AR", "05"),
        ("CA", "06"),
        ("CO", "08"),
        ("CT", "09"),
        ("DE", "10"),
        ("FL", "12"),
        ("GA", "13"),
        ("HI", "15"),
        ("ID", "16"),
        ("IL", "17"),
        ("IN", "18"),
        ("IA", "19"),
        ("KS", "20"),
        ("KY", "21"),
        ("LA", "22"),
        ("ME", "23"),
        ("MD", "24"),
        ("MA", "25"),
        ("MI", "26"),
        ("MN", "27"),
        ("MS", "28"),
        ("MO", "29"),
        ("MT", "30"),
        ("NE", "31"),
        ("NV", "32"),
        ("NH", "33"),
        ("NJ", "34"),
        ("NM", "35"),
        ("NY", "36"),
        ("NC", "37"),
        ("ND", "38"),
        ("OH", "39"),
        ("OK", "40"),
        ("OR", "41"),
        ("PA", "42"),
        ("RI", "44"),
        ("SC", "45"),
        ("SD", "46"),
        ("TN", "47"),
        ("TX", "48"),
        ("UT", "49"),
        ("VT", "50"),
        ("VA", "51"),
        ("WA", "53"),
        ("WV", "54"),
        ("WI", "55"),
        ("WY", "56"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economic_source_cli_names_roundtrip() {
        assert!(matches!(
            EconomicSource::from_cli_name("bls-laus"),
            Some(EconomicSource::BlsLaus)
        ));
        assert!(EconomicSource::from_cli_name("unknown").is_none());
    }

    #[test]
    fn base_geos_include_all_states_and_nation() {
        let geos = base_geos();
        assert_eq!(geos.len(), 52);
        assert!(geos.iter().any(|geo| geo.geo_id == "PA"));
        assert!(geos.iter().any(|geo| geo.geo_id == "US"));
    }

    #[test]
    fn state_fips_matches_state_count() {
        assert_eq!(state_fips().len(), 50);
        assert!(state_fips().contains(&("PA", "42")));
    }

    #[test]
    fn extracts_first_dollar_amount() {
        assert_eq!(
            extract_first_dollar_amount("$17.00 (NYC); $16.00 elsewhere"),
            Some(17.0)
        );
        assert_eq!(extract_first_dollar_amount(""), None);
    }

    #[test]
    fn parses_dol_embedded_minimum_wage_records() {
        let html = r#"
            vMinStateData = [
                { State: "PA", Wage: "$7.25", Type: "States with the same Minimum Wage as Federal", FootNote: "Y" },
                { State: "GA", Wage: "$5.15", Type: "States with lower Minimum Wage rates - Federal Applies", FootNote: "" },
                { State: "AL", Wage: "", Type: "States with no Minimum Wage rates - Federal Applies", FootNote: "" },
                { State: "WDC", Wage: "$17.95", Type: "States with Higher Minimum Wage than Federal", FootNote: "Y" },
            ];
        "#;

        let records = parse_dol_minimum_wage_records(html).expect("DOL records parse");

        assert_eq!(records.len(), 4);
        assert_eq!(records[0].state_code, "PA");
        assert_eq!(records[0].state_rate, Some(7.25));
        assert_eq!(records[0].effective_rate, 7.25);
        assert_eq!(records[1].state_rate, Some(5.15));
        assert_eq!(records[1].effective_rate, 7.25);
        assert_eq!(records[2].state_rate, None);
        assert_eq!(records[2].effective_rate, 7.25);
        assert_eq!(records[3].state_code, "DC");
    }

    #[test]
    fn parses_fhfa_state_hpi_yoy() {
        let body = "\
PA\t2025\t1\t267.36\n\
PA\t2026\t1\t277.52\n\
GA\t2025\t1\t325.09\n\
GA\t2026\t1\t325.45\n";

        let rows = parse_fhfa_state_hpi_yoy(body).expect("FHFA rows parse");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].0, "PA");
        assert_eq!(rows[1].1.year, 2026);
        assert_eq!(rows[1].1.quarter, 1);
        assert!((rows[1].2 - 3.8001197).abs() < 0.0001);
    }

    #[test]
    fn quarter_start_dates_match_calendar_quarters() {
        assert_eq!(
            quarter_start_date(2026, 3),
            NaiveDate::from_ymd_opt(2026, 7, 1)
        );
        assert_eq!(quarter_start_date(2026, 5), None);
    }

    #[test]
    fn calculates_bls_monthly_yoy_inputs() {
        let data = vec![
            BlsObservation {
                year: "2026".into(),
                period: "M05".into(),
                value: "321.047".into(),
            },
            BlsObservation {
                year: "2025".into(),
                period: "M05".into(),
                value: "312.607".into(),
            },
        ];

        let latest = latest_monthly_bls_observation(&data).expect("latest observation");
        let prior = same_month_prior_year_bls_observation(&data, latest).expect("prior year");
        let latest_value = latest.value.parse::<f64>().expect("latest value");
        let prior_value = prior.value.parse::<f64>().expect("prior value");
        let yoy = (latest_value / prior_value - 1.0) * 100.0;

        assert_eq!(
            bls_observation_date(latest),
            NaiveDate::from_ymd_opt(2026, 5, 1)
        );
        assert!((yoy - 2.6998).abs() < 0.001);
    }

    #[test]
    fn finds_previous_bls_monthly_observation() {
        let data = vec![
            BlsObservation {
                year: "2026".into(),
                period: "M01".into(),
                value: "159001".into(),
            },
            BlsObservation {
                year: "2025".into(),
                period: "M12".into(),
                value: "158829".into(),
            },
            BlsObservation {
                year: "2025".into(),
                period: "M11".into(),
                value: "158700".into(),
            },
        ];

        let latest = latest_monthly_bls_observation(&data).expect("latest observation");
        let previous = previous_monthly_bls_observation(&data, latest).expect("previous month");

        assert_eq!(previous.period, "M12");
    }

    #[test]
    fn parses_census_acs_state_metrics() {
        let body = r#"[
          ["NAME","B19013_001E","B25064_001E","B25070_001E","B25070_007E","B25070_008E","B25070_009E","B25070_010E","B17001_001E","B17001_002E","state"],
          ["Pennsylvania","75200","1210","1000","100","90","80","70","12000","1440","42"],
          ["District of Columbia","106000","1850","500","50","40","30","20","1000","120","11"]
        ]"#;

        let rows = parse_census_acs_state_metrics(body).expect("ACS rows parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].state_code, "PA");
        assert_eq!(rows[0].median_household_income, Some(75_200.0));
        assert_eq!(rows[0].median_gross_rent, Some(1_210.0));
        assert_eq!(rows[0].rent_burden_rate, Some(34.0));
        assert_eq!(rows[0].poverty_rate, Some(12.0));
    }

    #[test]
    fn parses_hud_fmr_state_medians() {
        let body = r#"{
          "data": [
            {"state_alpha": "PA", "fmr_2": "1450", "fmr_3": "1650"},
            {"state_alpha": "PA", "fmr_2": "1300", "fmr_3": "1500"},
            {"state_alpha": "PA", "fmr_2": "1380", "fmr_3": "1600"},
            {"state_alpha": "NY", "fmr_2": "2200", "fmr_3": "2500"},
            {"state_alpha": "NY", "fmr_2": "1900", "fmr_3": "2200"},
            {"state_code": "42", "fmr_2": "1200", "fmr_3": "1400"},
            {"state_code": "42", "fmr_2": "1400", "fmr_3": "1600"},
            {"state_alpha": "", "fmr_2": 999, "fmr_3": 999},
            {"no_state": true}
          ]
        }"#;

        let rows = parse_hud_fmr_state_medians(body).expect("HUD FMR rows parse");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "NY");
        assert!((rows[0].1 - 2050.0).abs() < 0.01);
        assert_eq!(rows[1].0, "PA");
        assert!((rows[1].1 - 1380.0).abs() < 0.01);
    }

    #[test]
    fn parses_eia_gas_price() {
        let body = r#"{
          "response": {
            "data": [
              {"period": "2026-06-16", "value": 3.189},
              {"period": "2026-06-09", "value": 3.175}
            ]
          }
        }"#;

        let price = parse_eia_latest_gas_price(body).expect("EIA gas price parse");
        assert_eq!(price, Some(3.189));
    }

    #[test]
    fn eia_gas_price_empty_data() {
        let body = r#"{"response": {"data": []}}"#;
        let price = parse_eia_latest_gas_price(body).expect("EIA empty data parse");
        assert_eq!(price, None);
    }

    #[test]
    fn parses_bea_state_gdp_response() {
        let body = r#"{
          "BEAAPI": {
            "Results": {
              "Data": [
                {"GeoFips": "PA", "DataValue": "856,789.123"},
                {"GeoFips": "NY", "DataValue": "1,964,321.456"},
                {"GeoFips": "02", "DataValue": ""},
                {"GeoFips": "TX", "DataValue": "1,904,039.000"}
              ]
            }
          }
        }"#;

        let rows = parse_bea_geofips_responses(body, "state_gdp").expect("BEA parse");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].0, "NY");
        assert!((rows[0].1 - 1_964_321_456_000.0).abs() < 1.0);
        assert_eq!(rows[1].0, "PA");
        assert!((rows[1].1 - 856_789_123_000.0).abs() < 1.0);
        assert_eq!(rows[2].0, "TX");
        assert!((rows[2].1 - 1_904_039_000_000.0).abs() < 1.0);
    }
}
