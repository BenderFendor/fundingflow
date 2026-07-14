use anyhow::Result;
use clap::{Parser, Subcommand};
use ingestion::Importer;
use sqlx::PgPool;
use tracing::info;
// This whole this shouldn't be a cli at all the api or the backend really should live pull and
// update the database with this infomation using the .env keys and stuff this shouldn't be a two
// part process but one thing
#[derive(Parser)]
#[command(name = "fundingflow")]
#[command(about = "FundingFlow CLI - manage and import public record data")]
struct Cli {
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Migrate,
    Seed,
    Import {
        #[arg(short, long)]
        source: String,
    },
    Derive {
        #[arg(short, long)]
        metric: String,
    },
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("running database migrations");
    db::run_migrations(pool).await?;
    info!("migrations complete");
    Ok(())
}

async fn run_seed(pool: &PgPool) -> Result<()> {
    info!("seeding sample data");

    let entity = db::queries::create_entity(
        pool,
        &domain::EntityType::Organization,
        "ACME Federal Contractors",
        "acme federal contractors",
    )
    .await?;
    info!(
        "created seed entity: {} ({})",
        entity.display_name, entity.id
    );

    let source_record = db::queries::create_source_record(
        pool,
        "usaspending",
        "award",
        "seed-award-001",
        Some("https://api.usaspending.gov/api/v2/awards/1/"),
        "abc123",
        None,
    )
    .await?;
    info!("created seed source record: {}", source_record.id);

    let evidence = db::queries::create_evidence(
        pool,
        source_record.id,
        Some("$.awarding_agency.toptier_agency.name"),
        Some("https://api.usaspending.gov/api/v2/awards/1/"),
        Some(1.0),
    )
    .await?;
    info!("created seed evidence: {}", evidence.id);

    db::queries::create_entity_identifier(
        pool,
        entity.id,
        &domain::IdentifierSource::Usaspending,
        &domain::IdentifierType::Uei,
        "SEEDUEI000001",
        Some(evidence.id),
    )
    .await?;
    info!("created seed entity identifier");

    db::queries::create_award(
        pool,
        &domain::CreateAwardRequest {
            generated_unique_award_id: "SEED-AWARD-001".to_string(),
            recipient_entity_id: Some(entity.id),
            awarding_agency_entity_id: None,
            funding_agency_entity_id: None,
            award_type: Some("contract".to_string()),
            description: Some("Sample federal contract for testing".to_string()),
            period_start: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            period_end: Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            obligation_amount: Some(1_500_000.0),
            outlay_amount: Some(1_200_000.0),
            place_of_performance: Some("Washington, DC".to_string()),
            recipient_geo_id: None,
            place_geo_id: None,
            evidence_id: Some(evidence.id),
        },
    )
    .await?;
    info!("created seed award");

    db::queries::create_relationship_edge(
        pool,
        &domain::CreateEdgeRequest {
            from_entity_id: entity.id,
            to_entity_id: entity.id,
            edge_type: domain::EdgeType::RecipientReceivedFederalObligation,
            started_on: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            ended_on: Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            amount: Some(1_500_000.0),
            currency: Some("USD".to_string()),
            description: Some("Sample federal obligation".to_string()),
            confidence: Some(1.0),
            evidence_id: Some(evidence.id),
            source: "usaspending".to_string(),
        },
    )
    .await?;
    info!("created seed relationship edge");

    info!("seed complete");
    Ok(())
}

async fn run_import(pool: &PgPool, source: &str) -> Result<()> {
    match source {
        "usaspending" => {
            let importer =
                ingestion::usaspending::UsaspendingImporter::new("https://api.usaspending.gov");
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "lda" => {
            let importer = ingestion::lda::LdaImporter::new("https://lda.gov");
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "fec" => {
            let importer =
                ingestion::fec::FecImporter::new("https://www.fec.gov/files/bulk-downloads", 2024);
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "openfec-schedule-e" => {
            let importer = ingestion::openfec_api::ScheduleEImporter::new(2024);
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "sec" => {
            let importer = ingestion::sec::SecImporter::new("https://data.sec.gov", 0);
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "rulemaking" => {
            let importer =
                ingestion::rulemaking::RulemakingImporter::new("https://www.federalregister.gov");
            let summary = importer.import(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        "economic-fixture" => {
            let summary = ingestion::economic::seed_fixture_state_metrics(pool).await?;
            info!(
                "import: {} records, {} entities, {} skipped, {} errors",
                summary.records_imported,
                summary.entities_created,
                summary.records_skipped,
                summary.errors
            );
        }
        other => {
            if let Some(source) = ingestion::economic::EconomicSource::from_cli_name(other) {
                let importer = ingestion::economic::EconomicImporter::new(source);
                let summary = importer.import(pool).await?;
                info!(
                    "import: {} records, {} entities, {} skipped, {} errors",
                    summary.records_imported,
                    summary.entities_created,
                    summary.records_skipped,
                    summary.errors
                );
            } else {
                anyhow::bail!("unknown source: {other}");
            }
        }
    }
    Ok(())
}

async fn run_derive(pool: &PgPool, metric: &str) -> Result<()> {
    match metric {
        "state-mvp" => {
            let summary = ingestion::economic::derive_state_mvp_metrics(pool).await?;
            info!(
                "derive: {} records, {} skipped, {} errors",
                summary.records_imported, summary.records_skipped, summary.errors
            );
        }
        other => anyhow::bail!("unknown derived metric set: {other}"),
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let database_url = cli.database_url.unwrap_or_else(db::default_database_url);
    let pool = db::create_pool(&database_url).await?;

    match cli.command {
        Command::Migrate => run_migrations(&pool).await?,
        Command::Seed => run_seed(&pool).await?,
        Command::Import { source } => run_import(&pool, &source).await?,
        Command::Derive { metric } => run_derive(&pool, &metric).await?,
    }

    Ok(())
}
