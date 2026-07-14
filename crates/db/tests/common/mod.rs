use domain::*;
use std::sync::OnceLock;
use uuid::Uuid;

static TEST_DB: OnceLock<String> = OnceLock::new();

pub fn test_db_url() -> &'static str {
    TEST_DB.get_or_init(|| {
        std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://fundingflow:fundingflow@localhost:5432/fundingflow".to_string()
        })
    })
}

pub struct TestSeed {
    pub companies: Vec<(Uuid, String)>,
    pub people: Vec<(Uuid, String)>,
    pub agencies: Vec<(Uuid, String)>,
    pub committees: Vec<(Uuid, String)>,
    pub evidence_id: Uuid,
}

pub async fn seed_test_data(pool: &sqlx::PgPool) -> TestSeed {
    sqlx::query("TRUNCATE TABLE entity, source_record CASCADE")
        .execute(pool)
        .await
        .unwrap();

    let mut seed = TestSeed {
        companies: Vec::new(),
        people: Vec::new(),
        agencies: Vec::new(),
        committees: Vec::new(),
        evidence_id: Uuid::nil(),
    };

    let run_id = Uuid::new_v4().to_string();
    let sr = db::queries::create_source_record(
        pool,
        "test",
        "seed",
        &format!("test-seed-{}", run_id),
        Some("test://seed"),
        "test-hash",
        None,
    )
    .await
    .unwrap();

    let ev =
        db::queries::create_evidence(pool, sr.id, Some("test"), Some("test://seed"), Some(1.0))
            .await
            .unwrap();
    seed.evidence_id = ev.id;

    let company_names = [
        "Tesla Inc",
        "SpaceX",
        "Palantir Technologies",
        "Lockheed Martin Corporation",
        "Boeing Company",
        "Raytheon Technologies",
        "Northrop Grumman",
        "General Dynamics",
        "Leidos Holdings",
        "Booz Allen Hamilton",
        "Amazon Web Services",
        "Microsoft Corporation",
        "Google LLC",
        "Oracle Corporation",
        "Salesforce Inc",
        "Cisco Systems",
        "Dell Technologies",
        "Intel Corporation",
        "NVIDIA Corporation",
        "Apple Inc",
    ];

    for name in &company_names {
        let canonical = name.to_lowercase();
        let entity = db::queries::create_entity(pool, &EntityType::Organization, name, &canonical)
            .await
            .unwrap();

        let _ = db::queries::create_entity_identifier(
            pool,
            entity.id,
            &IdentifierSource::Usaspending,
            &IdentifierType::Uei,
            &if *name == "Tesla Inc" {
                "UEI-TESLAINC".to_string()
            } else {
                format!(
                    "TEST-{}-{}",
                    name.to_uppercase().replace(' ', ""),
                    &run_id[..8]
                )
            },
            Some(ev.id),
        )
        .await;

        seed.companies.push((entity.id, name.to_string()));
    }

    let people_names = [
        "Elon Musk",
        "Jeff Bezos",
        "Tim Cook",
        "Satya Nadella",
        "Jensen Huang",
        "Mark Zuckerberg",
        "Sundar Pichai",
        "Andy Jassy",
        "Mary Barra",
        "Doug McMillon",
        "Brian Moynihan",
        "Jamie Dimon",
        "David Solomon",
        "Jane Fraser",
        "Arvind Krishna",
        "Ginni Rometty",
        "Michael Dell",
        "Lisa Su",
        "Pat Gelsinger",
        "Reed Hastings",
    ];

    for name in &people_names {
        let canonical = name.to_lowercase();
        let entity = db::queries::create_entity(pool, &EntityType::Person, name, &canonical)
            .await
            .unwrap();

        seed.people.push((entity.id, name.to_string()));
    }

    let agency_names = [
        "Department of Defense",
        "Department of Energy",
        "Department of Homeland Security",
        "Department of Health and Human Services",
        "National Aeronautics and Space Administration",
        "General Services Administration",
    ];

    for name in &agency_names {
        let canonical = name.to_lowercase();
        let entity = db::queries::create_entity(pool, &EntityType::Agency, name, &canonical)
            .await
            .unwrap();

        seed.agencies.push((entity.id, name.to_string()));
    }

    let committee_names = [
        "DNC Services Corp",
        "Republican National Committee",
        "Emily's List",
        "Club for Growth",
        "Majority PAC",
    ];

    for name in &committee_names {
        let canonical = name.to_lowercase();
        let entity = db::queries::create_entity(pool, &EntityType::Committee, name, &canonical)
            .await
            .unwrap();

        seed.committees.push((entity.id, name.to_string()));
    }

    for i in 0..company_names.len() {
        if i < people_names.len() {
            let (cid, _) = seed.companies[i];
            let (pid, _) = seed.people[i];

            let amount = 10_000.0 + (i as f64 * 250_000.0);

            let _ = db::queries::create_award(
                pool,
                &CreateAwardRequest {
                    generated_unique_award_id: format!("TEST-AWARD-{}-{}", run_id, i),
                    recipient_entity_id: Some(cid),
                    awarding_agency_entity_id: Some(seed.agencies[i % seed.agencies.len()].0),
                    funding_agency_entity_id: None,
                    award_type: Some("contract".to_string()),
                    description: Some(format!("Test award {} for company", i)),
                    period_start: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                    period_end: None,
                    obligation_amount: Some(amount),
                    outlay_amount: Some(amount * 0.8),
                    place_of_performance: Some("Test City, TS".to_string()),
                    recipient_geo_id: None,
                    place_geo_id: None,
                    evidence_id: Some(ev.id),
                },
            )
            .await
            .unwrap();

            let _ = db::queries::create_relationship_edge(
                pool,
                &CreateEdgeRequest {
                    from_entity_id: cid,
                    to_entity_id: seed.agencies[i % seed.agencies.len()].0,
                    edge_type: EdgeType::RecipientReceivedFederalObligation,
                    started_on: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                    ended_on: None,
                    amount: Some(amount),
                    currency: Some("USD".to_string()),
                    description: Some("Federal obligation".to_string()),
                    confidence: Some(1.0),
                    evidence_id: Some(ev.id),
                    source: "test".to_string(),
                },
            )
            .await
            .unwrap();

            let _ = db::queries::create_lobbying_filing(
                pool,
                &CreateLobbyingFilingRequest {
                    filing_uuid: format!("TEST-LDA-{}-{}", run_id, i),
                    registrant_entity_id: Some(cid),
                    client_entity_id: Some(cid),
                    filing_type: Some("Q".to_string()),
                    filing_period: Some("Q1".to_string()),
                    filing_year: Some(2025),
                    income_or_expense: Some("income".to_string()),
                    amount: Some(amount * 0.1),
                    issues_json: Some(serde_json::json!({
                        "count": 1,
                        "issues": [{"code": "DEF", "description": "Defense"}]
                    })),
                    evidence_id: Some(ev.id),
                },
            )
            .await
            .unwrap();

            let _ = db::queries::create_relationship_edge(
                pool,
                &CreateEdgeRequest {
                    from_entity_id: cid,
                    to_entity_id: pid,
                    edge_type: EdgeType::RegistrantLobbiedForClient,
                    started_on: None,
                    ended_on: None,
                    amount: Some(amount * 0.1),
                    currency: Some("USD".to_string()),
                    description: Some("Lobbying registration".to_string()),
                    confidence: Some(1.0),
                    evidence_id: Some(ev.id),
                    source: "test".to_string(),
                },
            )
            .await
            .unwrap();

            let _ = db::queries::create_relationship_edge(
                pool,
                &CreateEdgeRequest {
                    from_entity_id: pid,
                    to_entity_id: seed.committees[i % seed.committees.len()].0,
                    edge_type: EdgeType::PersonContributedToCommittee,
                    started_on: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                    ended_on: None,
                    amount: Some(2_900.0),
                    currency: Some("USD".to_string()),
                    description: Some("Individual contribution".to_string()),
                    confidence: Some(1.0),
                    evidence_id: Some(ev.id),
                    source: "test".to_string(),
                },
            )
            .await
            .unwrap();
        }
    }

    seed
}
