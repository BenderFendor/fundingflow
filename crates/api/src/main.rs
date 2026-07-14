use anyhow::Result;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderValue, Method, StatusCode, header},
    response::Json,
    routing::get,
};
use db::queries;
use domain::*;
use serde::Deserialize;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, signal};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};
use uuid::Uuid;

const MAX_QUERY_LENGTH: usize = 200;
const MAX_OFFSET: i64 = 1_000_000;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct FuzzySearchQuery {
    q: String,
    threshold: Option<f64>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct GeoSearchQuery {
    q: String,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct MetricsQuery {
    category: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct MetricHistoryQuery {
    from: Option<String>,
    to: Option<String>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct UnifiedSearchQuery {
    q: String,
    limit: Option<i64>,
    offset: Option<i64>,
}

fn validated_text(value: &str) -> Result<&str, StatusCode> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_QUERY_LENGTH {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

fn validated_limit(value: Option<i64>, default: i64, maximum: i64) -> Result<i64, StatusCode> {
    let value = value.unwrap_or(default);
    if !(1..=maximum).contains(&value) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

fn validated_offset(value: Option<i64>) -> Result<i64, StatusCode> {
    let value = value.unwrap_or(0);
    if !(0..=MAX_OFFSET).contains(&value) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

fn validated_threshold(value: Option<f64>) -> Result<f64, StatusCode> {
    let value = value.unwrap_or(0.15);
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

fn validated_cycle(value: Option<i32>) -> Result<Option<i32>, StatusCode> {
    match value {
        Some(cycle) if (1900..=2200).contains(&cycle) => Ok(Some(cycle)),
        Some(_) => Err(StatusCode::BAD_REQUEST),
        None => Ok(None),
    }
}

fn parse_optional_date(value: Option<&str>) -> Result<Option<chrono::NaiveDate>, StatusCode> {
    value
        .map(|value| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)
}

fn internal_error(error: anyhow::Error) -> StatusCode {
    error!(error = %error, "database query failed");
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "fundingflow-api"
    }))
}

async fn ready(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "ready",
            "database": "ok"
        }))),
        Err(error) => {
            error!(error = %error, "readiness check failed");
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "not_ready",
                    "database": "unavailable"
                })),
            ))
        }
    }
}

async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<EntityWithDetails>, StatusCode> {
    let entity = queries::get_entity(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let identifiers = queries::get_entity_identifiers(&state.pool, id)
        .await
        .map_err(internal_error)?;

    let aliases = queries::get_entity_aliases(&state.pool, id)
        .await
        .map_err(internal_error)?;

    Ok(Json(EntityWithDetails {
        entity,
        identifiers,
        aliases,
    }))
}

async fn search_entities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResult>, StatusCode> {
    let query = validated_text(&params.q)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let offset = validated_offset(params.offset)?;

    let result = queries::search_entities(&state.pool, query, limit, offset)
        .await
        .map_err(internal_error)?;

    Ok(Json(result))
}

async fn fuzzy_search_entities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FuzzySearchQuery>,
) -> Result<Json<Vec<EntitySummary>>, StatusCode> {
    let query = validated_text(&params.q)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let threshold = validated_threshold(params.threshold)?;

    let result = queries::fuzzy_search_entities(&state.pool, query, threshold, limit)
        .await
        .map_err(internal_error)?;

    Ok(Json(result))
}

async fn get_awards(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<Award>>, StatusCode> {
    let awards = queries::get_awards_for_entity(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(awards))
}

async fn get_lobbying(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<LobbyingFiling>>, StatusCode> {
    let filings = queries::get_lobbying_filings_for_entity(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(filings))
}

async fn get_edges(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<RelationshipEdge>>, StatusCode> {
    let edges = queries::get_edges_for_entity(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(edges))
}

async fn get_national_pulse(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NationalPulse>, StatusCode> {
    let pulse = queries::get_national_pulse(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(pulse))
}

async fn search_geos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GeoSearchQuery>,
) -> Result<Json<GeoSearchResult>, StatusCode> {
    let query = validated_text(&params.q)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let result = queries::search_geos(&state.pool, query, limit)
        .await
        .map_err(internal_error)?;

    Ok(Json(result))
}

async fn get_geo_profile(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
) -> Result<Json<StateProfile>, StatusCode> {
    let profile = queries::get_state_profile(&state.pool, &geo_id)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(profile))
}

async fn get_geo_metrics(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
    Query(params): Query<MetricsQuery>,
) -> Result<Json<Vec<MetricObservationWithDetails>>, StatusCode> {
    let limit = validated_limit(params.limit, 100, 500)?;
    let metrics = queries::get_latest_metric_observations(
        &state.pool,
        &geo_id,
        params.category.as_deref(),
        limit,
    )
    .await
    .map_err(internal_error)?;

    Ok(Json(metrics))
}

async fn get_geo_public_money(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
) -> Result<Json<PublicMoneySummary>, StatusCode> {
    let summary = queries::get_public_money_summary(&state.pool, &geo_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(summary))
}

async fn get_metric_history(
    State(state): State<Arc<AppState>>,
    Path((geo_id, metric_id)): Path<(String, String)>,
    Query(params): Query<MetricHistoryQuery>,
) -> Result<Json<Vec<MetricObservationWithDetails>>, StatusCode> {
    let limit = validated_limit(params.limit, 500, 2000)?;
    let from_date = parse_optional_date(params.from.as_deref())?;
    let to_date = parse_optional_date(params.to.as_deref())?;
    if matches!((from_date, to_date), (Some(from), Some(to)) if from > to) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let history =
        queries::get_metric_history(&state.pool, &geo_id, &metric_id, from_date, to_date, limit)
            .await
            .map_err(internal_error)?;

    Ok(Json(history))
}

async fn get_derived_metric_history(
    State(state): State<Arc<AppState>>,
    Path((geo_id, metric_id)): Path<(String, String)>,
    Query(params): Query<MetricHistoryQuery>,
) -> Result<Json<Vec<DerivedMetricObservation>>, StatusCode> {
    let limit = validated_limit(params.limit, 500, 2000)?;

    let history = queries::get_derived_metric_history(&state.pool, &geo_id, &metric_id, limit)
        .await
        .map_err(internal_error)?;

    Ok(Json(history))
}

async fn unified_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UnifiedSearchQuery>,
) -> Result<Json<UnifiedSearchResult>, StatusCode> {
    let query = validated_text(&params.q)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let offset = validated_offset(params.offset)?;

    let result = queries::unified_search(&state.pool, query, limit, offset)
        .await
        .map_err(internal_error)?;

    Ok(Json(result))
}

async fn get_entity_economic_context(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<StateProfile>>, StatusCode> {
    let profiles = queries::get_entity_economic_context(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(profiles))
}

#[derive(Deserialize)]
struct ContributionSearchQuery {
    q: String,
    cycle: Option<i32>,
    limit: Option<i64>,
}

async fn search_contributions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ContributionSearchQuery>,
) -> Result<Json<Vec<ContributionSearchResult>>, StatusCode> {
    let query = validated_text(&params.q)?;
    let cycle = validated_cycle(params.cycle)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let results = queries::search_contributions(&state.pool, query, cycle, limit)
        .await
        .map_err(internal_error)?;
    Ok(Json(results))
}

#[derive(Deserialize)]
struct EntityContributionsQuery {
    cycle: Option<i32>,
}

#[derive(Deserialize)]
struct ContributionFlowQuery {
    from_entity_id: Uuid,
    to_candidate_id: Uuid,
}

async fn get_entity_contributions(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
    Query(params): Query<EntityContributionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cycle = validated_cycle(params.cycle)?;
    let summary = queries::get_entity_contribution_summary(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    let transactions = queries::get_entity_transactions(&state.pool, entity_id, cycle, 500)
        .await
        .map_err(internal_error)?;

    let candidate_info = queries::get_candidate_info(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    let committee_info = queries::get_committee_info(&state.pool, entity_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(serde_json::json!({
        "summary": summary,
        "transactions": transactions,
        "candidate_info": candidate_info,
        "committee_info": committee_info,
    })))
}

async fn search_candidates_api(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<CandidateSearchResult>>, StatusCode> {
    let query = validated_text(&params.q)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let results = queries::search_candidates(&state.pool, query, limit)
        .await
        .map_err(internal_error)?;
    Ok(Json(results))
}

async fn get_contribution_flow(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ContributionFlowQuery>,
) -> Result<Json<MoneyFlowResult>, StatusCode> {
    if params.from_entity_id == params.to_candidate_id {
        return Err(StatusCode::BAD_REQUEST);
    }
    let flow = queries::get_money_flow_to_candidate(
        &state.pool,
        params.from_entity_id,
        params.to_candidate_id,
    )
    .await
    .map_err(internal_error)?;
    Ok(Json(flow))
}

#[derive(Deserialize)]
struct TopDonorsQuery {
    committee_id: String,
    limit: Option<i64>,
}

async fn get_top_donors(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TopDonorsQuery>,
) -> Result<Json<Vec<ContributionRecipientSummary>>, StatusCode> {
    let committee_id = validated_text(&params.committee_id)?;
    let limit = validated_limit(params.limit, 20, 100)?;
    let results = queries::get_top_donors_to_committee(&state.pool, committee_id, limit)
        .await
        .map_err(internal_error)?;
    Ok(Json(results))
}

fn cors_layer() -> CorsLayer {
    let allowed_origin =
        env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let layer = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE]);

    if allowed_origin == "*" {
        return layer.allow_origin(Any);
    }

    match allowed_origin.parse::<HeaderValue>() {
        Ok(origin) => layer.allow_origin(origin),
        Err(error) => {
            error!(%error, %allowed_origin, "invalid CORS_ALLOWED_ORIGIN; cross-origin requests disabled");
            layer
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://fundingflow:fundingflow@localhost:5432/fundingflow".to_string()
    });

    let pool = db::create_pool(&database_url).await?;
    let state = Arc::new(AppState { pool });

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/search", get(unified_search))
        .route("/api/v1/entities/search", get(search_entities))
        .route("/api/v1/entities/search/fuzzy", get(fuzzy_search_entities))
        .route("/api/v1/entities/{id}", get(get_entity))
        .route(
            "/api/v1/entities/{entity_id}/economic-context",
            get(get_entity_economic_context),
        )
        .route("/api/v1/entities/{entity_id}/awards", get(get_awards))
        .route("/api/v1/entities/{entity_id}/lobbying", get(get_lobbying))
        .route("/api/v1/entities/{entity_id}/edges", get(get_edges))
        .route("/api/v1/pulse/national", get(get_national_pulse))
        .route("/api/v1/geos/search", get(search_geos))
        .route("/api/v1/geos/{geo_id}/profile", get(get_geo_profile))
        .route("/api/v1/geos/{geo_id}/metrics", get(get_geo_metrics))
        .route(
            "/api/v1/geos/{geo_id}/public-money",
            get(get_geo_public_money),
        )
        .route(
            "/api/v1/geos/{geo_id}/metrics/{metric_id}/history",
            get(get_metric_history),
        )
        .route(
            "/api/v1/geos/{geo_id}/derived-metrics/{metric_id}/history",
            get(get_derived_metric_history),
        )
        .route("/api/v1/contributions/search", get(search_contributions))
        .route(
            "/api/v1/entities/{entity_id}/contributions",
            get(get_entity_contributions),
        )
        .route("/api/v1/candidates/search", get(search_candidates_api))
        .route("/api/v1/contributions/flow", get(get_contribution_flow))
        .route("/api/v1/contributions/top-donors", get(get_top_donors))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()?;

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("api server listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_validation_trims_and_rejects_empty_values() {
        assert_eq!(validated_text("  Boeing  ").unwrap(), "Boeing");
        assert_eq!(validated_text("   "), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn limit_validation_rejects_negative_zero_and_excessive_values() {
        assert_eq!(validated_limit(None, 20, 100).unwrap(), 20);
        assert_eq!(validated_limit(Some(1), 20, 100).unwrap(), 1);
        assert_eq!(validated_limit(Some(0), 20, 100), Err(StatusCode::BAD_REQUEST));
        assert_eq!(validated_limit(Some(-1), 20, 100), Err(StatusCode::BAD_REQUEST));
        assert_eq!(validated_limit(Some(101), 20, 100), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn offset_validation_rejects_negative_and_unbounded_values() {
        assert_eq!(validated_offset(None).unwrap(), 0);
        assert_eq!(validated_offset(Some(10)).unwrap(), 10);
        assert_eq!(validated_offset(Some(-1)), Err(StatusCode::BAD_REQUEST));
        assert_eq!(
            validated_offset(Some(MAX_OFFSET + 1)),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn threshold_validation_requires_a_finite_probability() {
        assert_eq!(validated_threshold(None).unwrap(), 0.15);
        assert_eq!(validated_threshold(Some(0.5)).unwrap(), 0.5);
        assert_eq!(
            validated_threshold(Some(f64::NAN)),
            Err(StatusCode::BAD_REQUEST)
        );
        assert_eq!(
            validated_threshold(Some(1.1)),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn date_validation_rejects_invalid_dates() {
        assert_eq!(
            parse_optional_date(Some("2026-07-14")).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 7, 14).unwrap())
        );
        assert_eq!(
            parse_optional_date(Some("07/14/2026")),
            Err(StatusCode::BAD_REQUEST)
        );
    }
}
