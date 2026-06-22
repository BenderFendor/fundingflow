use anyhow::Result;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use db::queries;
use domain::*;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

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

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<EntityWithDetails>, StatusCode> {
    let entity = queries::get_entity(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let identifiers = queries::get_entity_identifiers(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let aliases = queries::get_entity_aliases(&state.pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let result = queries::search_entities(&state.pool, &params.q, limit, offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn fuzzy_search_entities(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FuzzySearchQuery>,
) -> Result<Json<Vec<EntitySummary>>, StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let threshold = params.threshold.unwrap_or(0.15);

    let result = queries::fuzzy_search_entities(&state.pool, &params.q, threshold, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn get_awards(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<Award>>, StatusCode> {
    let awards = queries::get_awards_for_entity(&state.pool, entity_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(awards))
}

async fn get_lobbying(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<LobbyingFiling>>, StatusCode> {
    let filings = queries::get_lobbying_filings_for_entity(&state.pool, entity_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(filings))
}

async fn get_edges(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<RelationshipEdge>>, StatusCode> {
    let edges = queries::get_edges_for_entity(&state.pool, entity_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(edges))
}

async fn get_national_pulse(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NationalPulse>, StatusCode> {
    let pulse = queries::get_national_pulse(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(pulse))
}

async fn search_geos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GeoSearchQuery>,
) -> Result<Json<GeoSearchResult>, StatusCode> {
    let limit = params.limit.unwrap_or(20).min(100);
    let result = queries::search_geos(&state.pool, &params.q, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn get_geo_profile(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
) -> Result<Json<StateProfile>, StatusCode> {
    let profile = queries::get_state_profile(&state.pool, &geo_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(profile))
}

async fn get_geo_metrics(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
    Query(params): Query<MetricsQuery>,
) -> Result<Json<Vec<MetricObservationWithDetails>>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(500);
    let metrics = queries::get_latest_metric_observations(
        &state.pool,
        &geo_id,
        params.category.as_deref(),
        limit,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}

async fn get_geo_public_money(
    State(state): State<Arc<AppState>>,
    Path(geo_id): Path<String>,
) -> Result<Json<PublicMoneySummary>, StatusCode> {
    let summary = queries::get_public_money_summary(&state.pool, &geo_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(summary))
}

#[tokio::main]
async fn main() -> Result<()> {
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
        .route("/api/v1/entities/search", get(search_entities))
        .route("/api/v1/entities/search/fuzzy", get(fuzzy_search_entities))
        .route("/api/v1/entities/{id}", get(get_entity))
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
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()?;

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    info!("api server listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
