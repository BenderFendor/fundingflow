use anyhow::Result;
use domain::*;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn fuzzy_search_entities(
    pool: &PgPool,
    query: &str,
    threshold: f64,
    limit: i64,
) -> Result<Vec<EntitySummary>> {
    let rows = sqlx::query(
        r#"
        SELECT e.id, e.entity_type, e.display_name,
               COUNT(DISTINCT ei.id) as identifier_count,
               COUNT(DISTINCT a.id) as award_count,
               COALESCE(SUM(a.obligation_amount), 0.0) as total_obligations,
               similarity(e.display_name, $1) as sim
        FROM entity e
        LEFT JOIN entity_identifier ei ON ei.entity_id = e.id
        LEFT JOIN award a ON a.recipient_entity_id = e.id
        WHERE similarity(e.display_name, $1) > $2
           OR similarity(e.canonical_name, $1) > $2
        GROUP BY e.id, e.display_name, e.canonical_name
        ORDER BY sim DESC
        LIMIT $3
        "#,
    )
    .bind(query)
    .bind(threshold)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let entities = rows
        .iter()
        .map(|r| EntitySummary {
            id: r.get("id"),
            entity_type: r.get("entity_type"),
            display_name: r.get("display_name"),
            identifier_count: r.get("identifier_count"),
            award_count: r.get("award_count"),
            total_obligations: r.get("total_obligations"),
        })
        .collect();

    Ok(entities)
}

pub async fn create_entity(
    pool: &PgPool,
    entity_type: &EntityType,
    display_name: &str,
    canonical_name: &str,
) -> Result<Entity> {
    let row = sqlx::query(
        r#"
        INSERT INTO entity (entity_type, display_name, canonical_name)
        VALUES ($1, $2, $3)
        RETURNING id, entity_type, display_name, canonical_name, created_at, updated_at
        "#,
    )
    .bind(entity_type)
    .bind(display_name)
    .bind(canonical_name)
    .fetch_one(pool)
    .await?;

    Ok(Entity {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        display_name: row.get("display_name"),
        canonical_name: row.get("canonical_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_entity(pool: &PgPool, id: Uuid) -> Result<Option<Entity>> {
    let row = sqlx::query(
        "SELECT id, entity_type, display_name, canonical_name, created_at, updated_at FROM entity WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Entity {
        id: r.get("id"),
        entity_type: r.get("entity_type"),
        display_name: r.get("display_name"),
        canonical_name: r.get("canonical_name"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn get_entity_identifiers(
    pool: &PgPool,
    entity_id: Uuid,
) -> Result<Vec<EntityIdentifier>> {
    let rows = sqlx::query(
        "SELECT id, entity_id, source, identifier_type, identifier_value, valid_from, valid_to, evidence_id FROM entity_identifier WHERE entity_id = $1",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| {
            Ok(EntityIdentifier {
                id: r.get("id"),
                entity_id: r.get("entity_id"),
                source: r.get("source"),
                identifier_type: r.get("identifier_type"),
                identifier_value: r.get("identifier_value"),
                valid_from: r.get("valid_from"),
                valid_to: r.get("valid_to"),
                evidence_id: r.get("evidence_id"),
            })
        })
        .collect()
}

pub async fn get_entity_aliases(pool: &PgPool, entity_id: Uuid) -> Result<Vec<EntityAlias>> {
    let rows = sqlx::query(
        "SELECT id, entity_id, alias, normalized_alias, source, evidence_id FROM entity_alias WHERE entity_id = $1",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| {
            Ok(EntityAlias {
                id: r.get("id"),
                entity_id: r.get("entity_id"),
                alias: r.get("alias"),
                normalized_alias: r.get("normalized_alias"),
                source: r.get("source"),
                evidence_id: r.get("evidence_id"),
            })
        })
        .collect()
}

pub async fn search_entities(
    pool: &PgPool,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<SearchResult> {
    let search_pattern = format!("%{}%", query);

    let count_row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM entity WHERE display_name ILIKE $1 OR canonical_name ILIKE $1",
    )
    .bind(&search_pattern)
    .fetch_one(pool)
    .await?;
    let total: i64 = count_row.get("cnt");

    let rows = sqlx::query(
        r#"
        SELECT e.id, e.entity_type, e.display_name,
               (SELECT COUNT(id) FROM entity_identifier WHERE entity_id = e.id) as identifier_count,
               (
                   SELECT COUNT(DISTINCT a.id)
                   FROM award a
                   WHERE a.recipient_entity_id = e.id OR
                         a.recipient_entity_id IN (
                             SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = e.id
                             UNION
                             SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = e.id
                         )
               ) as award_count,
               (
                   SELECT SUM(a.obligation_amount)
                   FROM award a
                   WHERE a.recipient_entity_id = e.id OR
                         a.recipient_entity_id IN (
                             SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = e.id
                             UNION
                             SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = e.id
                         )
               ) as total_obligations
        FROM entity e
        WHERE e.display_name ILIKE $1 OR e.canonical_name ILIKE $1
        ORDER BY total_obligations DESC NULLS LAST, e.display_name
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let entities = rows
        .iter()
        .map(|r| EntitySummary {
            id: r.get("id"),
            entity_type: r.get("entity_type"),
            display_name: r.get("display_name"),
            identifier_count: r.get("identifier_count"),
            award_count: r.get("award_count"),
            total_obligations: r.get("total_obligations"),
        })
        .collect();

    Ok(SearchResult { entities, total })
}

pub async fn create_source_record(
    pool: &PgPool,
    source: &str,
    source_record_type: &str,
    source_record_id: &str,
    source_url: Option<&str>,
    content_hash: &str,
    raw_storage_path: Option<&str>,
) -> Result<SourceRecord> {
    let row = sqlx::query(
        r#"
        INSERT INTO source_record (source, source_record_type, source_record_id, source_url, content_hash, raw_storage_path)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, source, source_record_type, source_record_id, source_url, retrieved_at, content_hash, raw_storage_path
        "#,
    )
    .bind(source)
    .bind(source_record_type)
    .bind(source_record_id)
    .bind(source_url)
    .bind(content_hash)
    .bind(raw_storage_path)
    .fetch_one(pool)
    .await?;

    Ok(SourceRecord {
        id: row.get("id"),
        source: row.get("source"),
        source_record_type: row.get("source_record_type"),
        source_record_id: row.get("source_record_id"),
        source_url: row.get("source_url"),
        retrieved_at: row.get("retrieved_at"),
        content_hash: row.get("content_hash"),
        raw_storage_path: row.get("raw_storage_path"),
    })
}

pub async fn upsert_source_record(
    pool: &PgPool,
    source: &str,
    source_record_type: &str,
    source_record_id: &str,
    source_url: Option<&str>,
    content_hash: &str,
    raw_storage_path: Option<&str>,
) -> Result<SourceRecord> {
    let row = sqlx::query(
        r#"
        INSERT INTO source_record
            (source, source_record_type, source_record_id, source_url, content_hash, raw_storage_path)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (source, source_record_type, source_record_id) DO UPDATE SET
            source_url = EXCLUDED.source_url,
            retrieved_at = now(),
            content_hash = EXCLUDED.content_hash,
            raw_storage_path = EXCLUDED.raw_storage_path
        RETURNING id, source, source_record_type, source_record_id, source_url, retrieved_at, content_hash, raw_storage_path
        "#,
    )
    .bind(source)
    .bind(source_record_type)
    .bind(source_record_id)
    .bind(source_url)
    .bind(content_hash)
    .bind(raw_storage_path)
    .fetch_one(pool)
    .await?;

    Ok(SourceRecord {
        id: row.get("id"),
        source: row.get("source"),
        source_record_type: row.get("source_record_type"),
        source_record_id: row.get("source_record_id"),
        source_url: row.get("source_url"),
        retrieved_at: row.get("retrieved_at"),
        content_hash: row.get("content_hash"),
        raw_storage_path: row.get("raw_storage_path"),
    })
}

pub async fn create_evidence(
    pool: &PgPool,
    source_record_id: Uuid,
    quote_or_field_path: Option<&str>,
    source_url: Option<&str>,
    confidence: Option<f64>,
) -> Result<Evidence> {
    let row = sqlx::query(
        r#"
        INSERT INTO evidence (source_record_id, quote_or_field_path, source_url, confidence)
        VALUES ($1, $2, $3, $4)
        RETURNING id, source_record_id, quote_or_field_path, source_url, retrieved_at, confidence
        "#,
    )
    .bind(source_record_id)
    .bind(quote_or_field_path)
    .bind(source_url)
    .bind(confidence)
    .fetch_one(pool)
    .await?;

    Ok(Evidence {
        id: row.get("id"),
        source_record_id: row.get("source_record_id"),
        quote_or_field_path: row.get("quote_or_field_path"),
        source_url: row.get("source_url"),
        retrieved_at: row.get("retrieved_at"),
        confidence: row.get("confidence"),
    })
}

pub async fn create_entity_identifier(
    pool: &PgPool,
    entity_id: Uuid,
    source: &IdentifierSource,
    identifier_type: &IdentifierType,
    identifier_value: &str,
    evidence_id: Option<Uuid>,
) -> Result<EntityIdentifier> {
    let row = sqlx::query(
        r#"
        INSERT INTO entity_identifier (entity_id, source, identifier_type, identifier_value, evidence_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, entity_id, source, identifier_type, identifier_value, valid_from, valid_to, evidence_id
        "#,
    )
    .bind(entity_id)
    .bind(source)
    .bind(identifier_type)
    .bind(identifier_value)
    .bind(evidence_id)
    .fetch_one(pool)
    .await?;

    Ok(EntityIdentifier {
        id: row.get("id"),
        entity_id: row.get("entity_id"),
        source: row.get("source"),
        identifier_type: row.get("identifier_type"),
        identifier_value: row.get("identifier_value"),
        valid_from: row.get("valid_from"),
        valid_to: row.get("valid_to"),
        evidence_id: row.get("evidence_id"),
    })
}

pub async fn create_entity_alias(
    pool: &PgPool,
    entity_id: Uuid,
    alias: &str,
    normalized_alias: &str,
    source: &IdentifierSource,
    evidence_id: Option<Uuid>,
) -> Result<EntityAlias> {
    let row = sqlx::query(
        r#"
        INSERT INTO entity_alias (entity_id, alias, normalized_alias, source, evidence_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, entity_id, alias, normalized_alias, source, evidence_id
        "#,
    )
    .bind(entity_id)
    .bind(alias)
    .bind(normalized_alias)
    .bind(source)
    .bind(evidence_id)
    .fetch_one(pool)
    .await?;

    Ok(EntityAlias {
        id: row.get("id"),
        entity_id: row.get("entity_id"),
        alias: row.get("alias"),
        normalized_alias: row.get("normalized_alias"),
        source: row.get("source"),
        evidence_id: row.get("evidence_id"),
    })
}

pub async fn create_relationship_edge(
    pool: &PgPool,
    req: &CreateEdgeRequest,
) -> Result<RelationshipEdge> {
    let row = sqlx::query(
        r#"
        INSERT INTO relationship_edge (from_entity_id, to_entity_id, edge_type, started_on, ended_on, amount, currency, description, confidence, evidence_id, source)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, from_entity_id, to_entity_id, edge_type, started_on, ended_on, amount, currency, description, confidence, evidence_id, source
        "#,
    )
    .bind(req.from_entity_id)
    .bind(req.to_entity_id)
    .bind(&req.edge_type)
    .bind(req.started_on)
    .bind(req.ended_on)
    .bind(req.amount)
    .bind(req.currency.as_deref())
    .bind(req.description.as_deref())
    .bind(req.confidence)
    .bind(req.evidence_id)
    .bind(req.source.as_str())
    .fetch_one(pool)
    .await?;

    Ok(RelationshipEdge {
        id: row.get("id"),
        from_entity_id: row.get("from_entity_id"),
        to_entity_id: row.get("to_entity_id"),
        edge_type: row.get("edge_type"),
        started_on: row.get("started_on"),
        ended_on: row.get("ended_on"),
        amount: row.get("amount"),
        currency: row.get("currency"),
        description: row.get("description"),
        confidence: row.get("confidence"),
        evidence_id: row.get("evidence_id"),
        source: row.get("source"),
    })
}

pub async fn get_awards_for_entity(pool: &PgPool, entity_id: Uuid) -> Result<Vec<Award>> {
    let rows = sqlx::query(
        r#"
        SELECT id, generated_unique_award_id, recipient_entity_id, awarding_agency_entity_id,
               funding_agency_entity_id, award_type, description, period_start, period_end,
               obligation_amount, outlay_amount, place_of_performance, recipient_geo_id,
               place_geo_id, evidence_id
        FROM award
        WHERE recipient_entity_id = $1 OR
              recipient_entity_id IN (
                  SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = $1
                  UNION
                  SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = $1
              )
        ORDER BY period_start DESC NULLS LAST
        LIMIT 100
        "#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| {
            Ok(Award {
                id: r.get("id"),
                generated_unique_award_id: r.get("generated_unique_award_id"),
                recipient_entity_id: r.get("recipient_entity_id"),
                awarding_agency_entity_id: r.get("awarding_agency_entity_id"),
                funding_agency_entity_id: r.get("funding_agency_entity_id"),
                award_type: r.get("award_type"),
                description: r.get("description"),
                period_start: r.get("period_start"),
                period_end: r.get("period_end"),
                obligation_amount: r.get("obligation_amount"),
                outlay_amount: r.get("outlay_amount"),
                place_of_performance: r.get("place_of_performance"),
                recipient_geo_id: r.get("recipient_geo_id"),
                place_geo_id: r.get("place_geo_id"),
                evidence_id: r.get("evidence_id"),
            })
        })
        .collect()
}

pub async fn get_lobbying_filings_for_entity(
    pool: &PgPool,
    entity_id: Uuid,
) -> Result<Vec<LobbyingFiling>> {
    let rows = sqlx::query(
        r#"
        SELECT id, filing_uuid, registrant_entity_id, client_entity_id, filing_type,
               filing_period, filing_year, income_or_expense, amount, issues_json, evidence_id
        FROM lobbying_filing
        WHERE registrant_entity_id = $1 OR client_entity_id = $1 OR
              registrant_entity_id IN (
                  SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = $1
                  UNION
                  SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = $1
              ) OR
              client_entity_id IN (
                  SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = $1
                  UNION
                  SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = $1
              )
        ORDER BY filing_year DESC NULLS LAST
        LIMIT 100
        "#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| {
            Ok(LobbyingFiling {
                id: r.get("id"),
                filing_uuid: r.get("filing_uuid"),
                registrant_entity_id: r.get("registrant_entity_id"),
                client_entity_id: r.get("client_entity_id"),
                filing_type: r.get("filing_type"),
                filing_period: r.get("filing_period"),
                filing_year: r.get("filing_year"),
                income_or_expense: r.get("income_or_expense"),
                amount: r.get("amount"),
                issues_json: r.get("issues_json"),
                evidence_id: r.get("evidence_id"),
            })
        })
        .collect()
}

pub async fn get_edges_for_entity(pool: &PgPool, entity_id: Uuid) -> Result<Vec<RelationshipEdge>> {
    let rows = sqlx::query(
        r#"
        SELECT id, from_entity_id, to_entity_id, edge_type, started_on, ended_on,
               amount, currency, description, confidence, evidence_id, source
        FROM relationship_edge
        WHERE from_entity_id = $1 OR to_entity_id = $1
        ORDER BY started_on DESC NULLS LAST
        LIMIT 200
        "#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| {
            Ok(RelationshipEdge {
                id: r.get("id"),
                from_entity_id: r.get("from_entity_id"),
                to_entity_id: r.get("to_entity_id"),
                edge_type: r.get("edge_type"),
                started_on: r.get("started_on"),
                ended_on: r.get("ended_on"),
                amount: r.get("amount"),
                currency: r.get("currency"),
                description: r.get("description"),
                confidence: r.get("confidence"),
                evidence_id: r.get("evidence_id"),
                source: r.get("source"),
            })
        })
        .collect()
}

pub async fn find_entity_by_identifier(
    pool: &PgPool,
    source: &IdentifierSource,
    identifier_type: &IdentifierType,
    identifier_value: &str,
) -> Result<Option<Entity>> {
    let row = sqlx::query(
        r#"
        SELECT e.id, e.entity_type, e.display_name, e.canonical_name, e.created_at, e.updated_at
        FROM entity e
        JOIN entity_identifier ei ON ei.entity_id = e.id
        WHERE ei.source = $1 AND ei.identifier_type = $2 AND ei.identifier_value = $3
        LIMIT 1
        "#,
    )
    .bind(source)
    .bind(identifier_type)
    .bind(identifier_value)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Entity {
        id: r.get("id"),
        entity_type: r.get("entity_type"),
        display_name: r.get("display_name"),
        canonical_name: r.get("canonical_name"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn find_entity_by_name(pool: &PgPool, canonical_name: &str) -> Result<Option<Entity>> {
    let row = sqlx::query(
        "SELECT id, entity_type, display_name, canonical_name, created_at, updated_at FROM entity WHERE canonical_name = $1",
    )
    .bind(canonical_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Entity {
        id: r.get("id"),
        entity_type: r.get("entity_type"),
        display_name: r.get("display_name"),
        canonical_name: r.get("canonical_name"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

pub async fn find_entities_by_name_fuzzy(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<Entity>> {
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, entity_type, display_name, canonical_name, created_at, updated_at FROM entity WHERE display_name ILIKE $1 OR canonical_name ILIKE $1 ORDER BY similarity(display_name, $2) DESC LIMIT $3",
    )
    .bind(&search_pattern)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| Entity {
            id: r.get("id"),
            entity_type: r.get("entity_type"),
            display_name: r.get("display_name"),
            canonical_name: r.get("canonical_name"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect())
}

pub async fn create_award(pool: &PgPool, req: &CreateAwardRequest) -> Result<Award> {
    let row = sqlx::query(
        r#"
        INSERT INTO award (generated_unique_award_id, recipient_entity_id, awarding_agency_entity_id,
                          funding_agency_entity_id, award_type, description, period_start, period_end,
                          obligation_amount, outlay_amount, place_of_performance, recipient_geo_id,
                          place_geo_id, evidence_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING id, generated_unique_award_id, recipient_entity_id, awarding_agency_entity_id,
                  funding_agency_entity_id, award_type, description, period_start, period_end,
                  obligation_amount, outlay_amount, place_of_performance, recipient_geo_id,
                  place_geo_id, evidence_id
        "#,
    )
    .bind(&req.generated_unique_award_id)
    .bind(req.recipient_entity_id)
    .bind(req.awarding_agency_entity_id)
    .bind(req.funding_agency_entity_id)
    .bind(req.award_type.as_deref())
    .bind(req.description.as_deref())
    .bind(req.period_start)
    .bind(req.period_end)
    .bind(req.obligation_amount)
    .bind(req.outlay_amount)
    .bind(req.place_of_performance.as_deref())
    .bind(req.recipient_geo_id.as_deref())
    .bind(req.place_geo_id.as_deref())
    .bind(req.evidence_id)
    .fetch_one(pool)
    .await?;

    Ok(Award {
        id: row.get("id"),
        generated_unique_award_id: row.get("generated_unique_award_id"),
        recipient_entity_id: row.get("recipient_entity_id"),
        awarding_agency_entity_id: row.get("awarding_agency_entity_id"),
        funding_agency_entity_id: row.get("funding_agency_entity_id"),
        award_type: row.get("award_type"),
        description: row.get("description"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        obligation_amount: row.get("obligation_amount"),
        outlay_amount: row.get("outlay_amount"),
        place_of_performance: row.get("place_of_performance"),
        recipient_geo_id: row.get("recipient_geo_id"),
        place_geo_id: row.get("place_geo_id"),
        evidence_id: row.get("evidence_id"),
    })
}

pub async fn create_lobbying_filing(
    pool: &PgPool,
    req: &CreateLobbyingFilingRequest,
) -> Result<LobbyingFiling> {
    let row = sqlx::query(
        r#"
        INSERT INTO lobbying_filing (filing_uuid, registrant_entity_id, client_entity_id, filing_type,
                                     filing_period, filing_year, income_or_expense, amount, issues_json, evidence_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, filing_uuid, registrant_entity_id, client_entity_id, filing_type,
                  filing_period, filing_year, income_or_expense, amount, issues_json, evidence_id
        "#,
    )
    .bind(&req.filing_uuid)
    .bind(req.registrant_entity_id)
    .bind(req.client_entity_id)
    .bind(req.filing_type.as_deref())
    .bind(req.filing_period.as_deref())
    .bind(req.filing_year)
    .bind(req.income_or_expense.as_deref())
    .bind(req.amount)
    .bind(&req.issues_json)
    .bind(req.evidence_id)
    .fetch_one(pool)
    .await?;

    Ok(LobbyingFiling {
        id: row.get("id"),
        filing_uuid: row.get("filing_uuid"),
        registrant_entity_id: row.get("registrant_entity_id"),
        client_entity_id: row.get("client_entity_id"),
        filing_type: row.get("filing_type"),
        filing_period: row.get("filing_period"),
        filing_year: row.get("filing_year"),
        income_or_expense: row.get("income_or_expense"),
        amount: row.get("amount"),
        issues_json: row.get("issues_json"),
        evidence_id: row.get("evidence_id"),
    })
}

pub async fn upsert_geo(pool: &PgPool, geo: &Geo) -> Result<Geo> {
    let row = sqlx::query(
        r#"
        INSERT INTO geo (geo_id, geo_type, name, state_code, county_code, region)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (geo_id) DO UPDATE SET
            geo_type = EXCLUDED.geo_type,
            name = EXCLUDED.name,
            state_code = EXCLUDED.state_code,
            county_code = EXCLUDED.county_code,
            region = EXCLUDED.region
        RETURNING geo_id, geo_type, name, state_code, county_code, region
        "#,
    )
    .bind(&geo.geo_id)
    .bind(&geo.geo_type)
    .bind(&geo.name)
    .bind(geo.state_code.as_deref())
    .bind(geo.county_code.as_deref())
    .bind(geo.region.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(row_to_geo(&row))
}

pub async fn upsert_data_source(pool: &PgPool, source: &DataSource) -> Result<DataSource> {
    let row = sqlx::query(
        r#"
        INSERT INTO data_source
            (source_id, agency, dataset, url, license, update_frequency, requires_api_key, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (source_id) DO UPDATE SET
            agency = EXCLUDED.agency,
            dataset = EXCLUDED.dataset,
            url = EXCLUDED.url,
            license = EXCLUDED.license,
            update_frequency = EXCLUDED.update_frequency,
            requires_api_key = EXCLUDED.requires_api_key,
            notes = EXCLUDED.notes
        RETURNING source_id, agency, dataset, url, license, update_frequency, requires_api_key, notes
        "#,
    )
    .bind(&source.source_id)
    .bind(&source.agency)
    .bind(&source.dataset)
    .bind(&source.url)
    .bind(source.license.as_deref())
    .bind(source.update_frequency.as_deref())
    .bind(source.requires_api_key)
    .bind(source.notes.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(row_to_data_source(&row))
}

pub async fn upsert_metric(pool: &PgPool, metric: &Metric) -> Result<Metric> {
    let row = sqlx::query(
        r#"
        INSERT INTO metric
            (metric_id, name, category, unit, seasonal_adjustment, source_id, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (metric_id) DO UPDATE SET
            name = EXCLUDED.name,
            category = EXCLUDED.category,
            unit = EXCLUDED.unit,
            seasonal_adjustment = EXCLUDED.seasonal_adjustment,
            source_id = EXCLUDED.source_id,
            notes = EXCLUDED.notes
        RETURNING metric_id, name, category, unit, seasonal_adjustment, source_id, notes
        "#,
    )
    .bind(&metric.metric_id)
    .bind(&metric.name)
    .bind(&metric.category)
    .bind(&metric.unit)
    .bind(metric.seasonal_adjustment.as_deref())
    .bind(&metric.source_id)
    .bind(metric.notes.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(row_to_metric(&row))
}

pub async fn upsert_metric_observation(
    pool: &PgPool,
    observation: &MetricObservation,
) -> Result<MetricObservation> {
    let row = sqlx::query(
        r#"
        INSERT INTO metric_observation
            (metric_id, geo_id, date, value, vintage_date, release_date, source_series_id, evidence_id, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (metric_id, geo_id, date, vintage_date) DO UPDATE SET
            value = EXCLUDED.value,
            release_date = EXCLUDED.release_date,
            source_series_id = EXCLUDED.source_series_id,
            evidence_id = EXCLUDED.evidence_id,
            notes = EXCLUDED.notes
        RETURNING metric_id, geo_id, date, value, vintage_date, release_date, source_series_id, evidence_id, notes
        "#,
    )
    .bind(&observation.metric_id)
    .bind(&observation.geo_id)
    .bind(observation.date)
    .bind(observation.value)
    .bind(observation.vintage_date)
    .bind(observation.release_date)
    .bind(observation.source_series_id.as_deref())
    .bind(observation.evidence_id)
    .bind(observation.notes.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(row_to_metric_observation(&row))
}

pub async fn upsert_derived_metric_observation(
    pool: &PgPool,
    observation: &DerivedMetricObservation,
) -> Result<DerivedMetricObservation> {
    let row = sqlx::query(
        r#"
        INSERT INTO derived_metric_observation
            (metric_id, geo_id, date, value, formula, input_metric_ids, vintage_date, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (metric_id, geo_id, date, vintage_date) DO UPDATE SET
            value = EXCLUDED.value,
            formula = EXCLUDED.formula,
            input_metric_ids = EXCLUDED.input_metric_ids,
            notes = EXCLUDED.notes
        RETURNING metric_id, geo_id, date, value, formula, input_metric_ids, vintage_date, notes
        "#,
    )
    .bind(&observation.metric_id)
    .bind(&observation.geo_id)
    .bind(observation.date)
    .bind(observation.value)
    .bind(&observation.formula)
    .bind(&observation.input_metric_ids)
    .bind(observation.vintage_date)
    .bind(observation.notes.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(row_to_derived_metric_observation(&row))
}

pub async fn search_geos(pool: &PgPool, query: &str, limit: i64) -> Result<GeoSearchResult> {
    let pattern = format!("%{}%", query);
    let count_row =
        sqlx::query("SELECT COUNT(*) AS cnt FROM geo WHERE name ILIKE $1 OR geo_id ILIKE $1")
            .bind(&pattern)
            .fetch_one(pool)
            .await?;
    let total = count_row.get("cnt");

    let rows = sqlx::query(
        r#"
        SELECT geo_id, geo_type, name, state_code, county_code, region
        FROM geo
        WHERE name ILIKE $1 OR geo_id ILIKE $1
        ORDER BY
            CASE geo_type WHEN 'nation' THEN 0 WHEN 'state' THEN 1 ELSE 2 END,
            name
        LIMIT $2
        "#,
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(GeoSearchResult {
        geos: rows.iter().map(row_to_geo).collect(),
        total,
    })
}

pub async fn get_geo(pool: &PgPool, geo_id: &str) -> Result<Option<Geo>> {
    let row = sqlx::query(
        "SELECT geo_id, geo_type, name, state_code, county_code, region FROM geo WHERE geo_id = $1",
    )
    .bind(geo_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.as_ref().map(row_to_geo))
}

pub async fn get_latest_metric_observations(
    pool: &PgPool,
    geo_id: &str,
    category: Option<&str>,
    limit: i64,
) -> Result<Vec<MetricObservationWithDetails>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (mo.metric_id)
            mo.metric_id, mo.geo_id, mo.date, mo.value, mo.vintage_date, mo.release_date,
            mo.source_series_id, mo.evidence_id, mo.notes AS observation_notes,
            m.name AS metric_name, m.category, m.unit, m.seasonal_adjustment,
            m.source_id, m.notes AS metric_notes,
            ds.agency, ds.dataset, ds.url, ds.license, ds.update_frequency,
            ds.requires_api_key, ds.notes AS source_notes
        FROM metric_observation mo
        JOIN metric m ON m.metric_id = mo.metric_id
        JOIN data_source ds ON ds.source_id = m.source_id
        WHERE mo.geo_id = $1 AND ($2::text IS NULL OR m.category = $2)
        ORDER BY mo.metric_id, mo.date DESC, mo.vintage_date DESC
        LIMIT $3
        "#,
    )
    .bind(geo_id)
    .bind(category)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(row_to_metric_observation_with_details)
        .collect())
}

pub async fn get_latest_derived_metrics(
    pool: &PgPool,
    geo_id: &str,
) -> Result<Vec<DerivedMetricObservation>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (metric_id)
            metric_id, geo_id, date, value, formula, input_metric_ids, vintage_date, notes
        FROM derived_metric_observation
        WHERE geo_id = $1
        ORDER BY metric_id, date DESC, vintage_date DESC
        "#,
    )
    .bind(geo_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(row_to_derived_metric_observation).collect())
}

pub async fn get_public_money_summary(pool: &PgPool, geo_id: &str) -> Result<PublicMoneySummary> {
    let summary_row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(obligation_amount), 0.0) AS total_obligations,
               COUNT(*) AS award_count
        FROM award
        WHERE place_geo_id = $1 OR recipient_geo_id = $1
        "#,
    )
    .bind(geo_id)
    .fetch_one(pool)
    .await?;

    let recipient_rows = sqlx::query(
        r#"
        SELECT a.recipient_entity_id,
               COALESCE(e.display_name, 'Unknown Recipient') AS display_name,
               COALESCE(SUM(a.obligation_amount), 0.0) AS total_obligations,
               COUNT(*) AS award_count
        FROM award a
        LEFT JOIN entity e ON e.id = a.recipient_entity_id
        WHERE a.place_geo_id = $1 OR a.recipient_geo_id = $1
        GROUP BY a.recipient_entity_id, e.display_name
        ORDER BY total_obligations DESC
        LIMIT 10
        "#,
    )
    .bind(geo_id)
    .fetch_all(pool)
    .await?;

    Ok(PublicMoneySummary {
        geo_id: geo_id.to_string(),
        total_obligations: summary_row.get("total_obligations"),
        award_count: summary_row.get("award_count"),
        top_recipients: recipient_rows
            .iter()
            .map(|row| PublicMoneyRecipient {
                entity_id: row.get("recipient_entity_id"),
                display_name: row.get("display_name"),
                total_obligations: row.get("total_obligations"),
                award_count: row.get("award_count"),
            })
            .collect(),
    })
}

pub async fn get_state_profile(pool: &PgPool, geo_id: &str) -> Result<Option<StateProfile>> {
    let Some(geo) = get_geo(pool, geo_id).await? else {
        return Ok(None);
    };

    let latest_metrics = get_latest_metric_observations(pool, geo_id, None, 100).await?;
    let derived_metrics = get_latest_derived_metrics(pool, geo_id).await?;
    let public_money = get_public_money_summary(pool, geo_id).await?;

    Ok(Some(StateProfile {
        geo,
        latest_metrics,
        derived_metrics,
        public_money,
    }))
}

pub async fn get_national_pulse(pool: &PgPool) -> Result<Option<NationalPulse>> {
    let Some(geo) = get_geo(pool, "US").await? else {
        return Ok(None);
    };

    let latest_metrics = get_latest_metric_observations(pool, "US", None, 30).await?;
    let derived_metrics = get_latest_derived_metrics(pool, "US").await?;

    Ok(Some(NationalPulse {
        geo,
        latest_metrics,
        derived_metrics,
    }))
}

fn row_to_geo(row: &sqlx::postgres::PgRow) -> Geo {
    Geo {
        geo_id: row.get("geo_id"),
        geo_type: row.get("geo_type"),
        name: row.get("name"),
        state_code: row.get("state_code"),
        county_code: row.get("county_code"),
        region: row.get("region"),
    }
}

fn row_to_data_source(row: &sqlx::postgres::PgRow) -> DataSource {
    DataSource {
        source_id: row.get("source_id"),
        agency: row.get("agency"),
        dataset: row.get("dataset"),
        url: row.get("url"),
        license: row.get("license"),
        update_frequency: row.get("update_frequency"),
        requires_api_key: row.get("requires_api_key"),
        notes: row.get("notes"),
    }
}

fn row_to_metric(row: &sqlx::postgres::PgRow) -> Metric {
    Metric {
        metric_id: row.get("metric_id"),
        name: row.get("name"),
        category: row.get("category"),
        unit: row.get("unit"),
        seasonal_adjustment: row.get("seasonal_adjustment"),
        source_id: row.get("source_id"),
        notes: row.get("notes"),
    }
}

fn row_to_metric_observation(row: &sqlx::postgres::PgRow) -> MetricObservation {
    MetricObservation {
        metric_id: row.get("metric_id"),
        geo_id: row.get("geo_id"),
        date: row.get("date"),
        value: row.get("value"),
        vintage_date: row.get("vintage_date"),
        release_date: row.get("release_date"),
        source_series_id: row.get("source_series_id"),
        evidence_id: row.get("evidence_id"),
        notes: row.get("notes"),
    }
}

fn row_to_derived_metric_observation(row: &sqlx::postgres::PgRow) -> DerivedMetricObservation {
    DerivedMetricObservation {
        metric_id: row.get("metric_id"),
        geo_id: row.get("geo_id"),
        date: row.get("date"),
        value: row.get("value"),
        formula: row.get("formula"),
        input_metric_ids: row.get("input_metric_ids"),
        vintage_date: row.get("vintage_date"),
        notes: row.get("notes"),
    }
}

fn row_to_metric_observation_with_details(
    row: &sqlx::postgres::PgRow,
) -> MetricObservationWithDetails {
    MetricObservationWithDetails {
        observation: MetricObservation {
            metric_id: row.get("metric_id"),
            geo_id: row.get("geo_id"),
            date: row.get("date"),
            value: row.get("value"),
            vintage_date: row.get("vintage_date"),
            release_date: row.get("release_date"),
            source_series_id: row.get("source_series_id"),
            evidence_id: row.get("evidence_id"),
            notes: row.get("observation_notes"),
        },
        metric: Metric {
            metric_id: row.get("metric_id"),
            name: row.get("metric_name"),
            category: row.get("category"),
            unit: row.get("unit"),
            seasonal_adjustment: row.get("seasonal_adjustment"),
            source_id: row.get("source_id"),
            notes: row.get("metric_notes"),
        },
        source: DataSource {
            source_id: row.get("source_id"),
            agency: row.get("agency"),
            dataset: row.get("dataset"),
            url: row.get("url"),
            license: row.get("license"),
            update_frequency: row.get("update_frequency"),
            requires_api_key: row.get("requires_api_key"),
            notes: row.get("source_notes"),
        },
    }
}
