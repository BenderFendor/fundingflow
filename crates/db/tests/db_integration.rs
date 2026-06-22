mod common;

use common::seed_test_data;
use domain::*;
use sqlx::PgPool;

async fn setup_pool() -> PgPool {
    let url = common::test_db_url();
    db::create_pool(url).await.unwrap()
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_seed_creates_all_entities() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    assert_eq!(_seed.companies.len(), 20);
    assert_eq!(_seed.people.len(), 20);
    assert_eq!(_seed.agencies.len(), 6);
    assert_eq!(_seed.committees.len(), 5);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_search_finds_entity() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let result = db::queries::search_entities(&pool, "Tesla", 10, 0)
        .await
        .unwrap();
    assert!(result.total >= 1);
    assert!(
        result
            .entities
            .iter()
            .any(|e| e.display_name == "Tesla Inc")
    );
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_fuzzy_search_typo() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let results = db::queries::fuzzy_search_entities(&pool, "Telsa Inc", 0.15, 10)
        .await
        .unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|e| e.display_name.contains("Tesla")));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_fuzzy_search_elon() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let results = db::queries::fuzzy_search_entities(&pool, "elon", 0.15, 10)
        .await
        .unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|e| e.display_name == "Elon Musk"));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_fuzzy_search_jef() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let results = db::queries::fuzzy_search_entities(&pool, "jef", 0.15, 10)
        .await
        .unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|e| e.display_name.contains("Jeff")));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_entity_with_details() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let (company_id, _) = &_seed.companies[0];
    let entity = db::queries::get_entity(&pool, *company_id)
        .await
        .unwrap()
        .unwrap();
    assert!(!entity.display_name.is_empty());

    let ids = db::queries::get_entity_identifiers(&pool, *company_id)
        .await
        .unwrap();
    assert!(!ids.is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_awards_for_entity() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let (company_id, _) = &_seed.companies[0];
    let awards = db::queries::get_awards_for_entity(&pool, *company_id)
        .await
        .unwrap();
    assert!(!awards.is_empty());
    assert!(awards[0].obligation_amount.unwrap() > 0.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_edges_for_entity() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let (company_id, _) = &_seed.companies[0];
    let edges = db::queries::get_edges_for_entity(&pool, *company_id)
        .await
        .unwrap();
    assert!(edges.len() >= 2);
    assert!(
        edges
            .iter()
            .any(|e| e.edge_type == EdgeType::RecipientReceivedFederalObligation)
    );
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_find_entity_by_identifier() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let entity = db::queries::find_entity_by_identifier(
        &pool,
        &IdentifierSource::Usaspending,
        &IdentifierType::Uei,
        "UEI-TESLAINC",
    )
    .await
    .unwrap();
    assert!(entity.is_some());
    assert_eq!(entity.unwrap().display_name, "Tesla Inc");
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_find_entity_by_name() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let entity = db::queries::find_entity_by_name(&pool, "tesla inc")
        .await
        .unwrap();
    assert!(entity.is_some());
    assert_eq!(entity.unwrap().display_name, "Tesla Inc");
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_lobbying_filings_for_entity() {
    let pool = setup_pool().await;
    let _seed = seed_test_data(&pool).await;

    let (company_id, _) = &_seed.companies[0];
    let filings = db::queries::get_lobbying_filings_for_entity(&pool, *company_id)
        .await
        .unwrap();
    assert!(!filings.is_empty());
    assert_eq!(filings[0].filing_type.as_deref(), Some("Q"));
}
