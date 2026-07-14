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
    let search_pattern = format!("%{}%", query);
    let rows = sqlx::query(
        r#"
        SELECT e.id, e.entity_type, e.display_name,
               COUNT(DISTINCT ei.id) as identifier_count,
               COUNT(DISTINCT a.id) as award_count,
               COALESCE(SUM(a.obligation_amount), 0.0) as total_obligations,
               GREATEST(
                   similarity(e.display_name, $1),
                   similarity(e.canonical_name, $1),
                   COALESCE((SELECT MAX(similarity(ea.alias, $1)) FROM entity_alias ea WHERE ea.entity_id = e.id), 0),
                   COALESCE((SELECT MAX(similarity(ei2.identifier_value, $1)) FROM entity_identifier ei2 WHERE ei2.entity_id = e.id), 0)
               ) as sim
        FROM entity e
        LEFT JOIN entity_identifier ei ON ei.entity_id = e.id
        LEFT JOIN award a ON a.recipient_entity_id = e.id
        WHERE similarity(e.display_name, $1) > $2
           OR similarity(e.canonical_name, $1) > $2
           OR e.display_name ILIKE $3 OR e.canonical_name ILIKE $3
           OR e.id IN (SELECT entity_id FROM entity_identifier WHERE identifier_value ILIKE $3)
           OR e.id IN (SELECT entity_id FROM entity_alias WHERE alias ILIKE $3)
        GROUP BY e.id, e.display_name, e.canonical_name
        ORDER BY sim DESC
        LIMIT $4
        "#,
    )
    .bind(query)
    .bind(threshold)
    .bind(&search_pattern)
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
        r#"
        SELECT COUNT(*) as cnt FROM entity
        WHERE display_name ILIKE $1 OR canonical_name ILIKE $1
           OR id IN (SELECT entity_id FROM entity_identifier WHERE identifier_value ILIKE $1)
           OR id IN (SELECT entity_id FROM entity_alias WHERE alias ILIKE $1)
        "#,
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
           OR e.id IN (SELECT entity_id FROM entity_identifier WHERE identifier_value ILIKE $1)
           OR e.id IN (SELECT entity_id FROM entity_alias WHERE alias ILIKE $1)
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

pub async fn get_linked_geo_ids(pool: &PgPool, entity_id: Uuid) -> Result<Vec<String>> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT geo_id
        FROM (
            SELECT unnest(ARRAY[
                NULLIF(a.recipient_geo_id, ''),
                NULLIF(a.place_geo_id, ''),
                NULLIF(SUBSTRING(a.place_of_performance FROM ', ([A-Z]{2})$'), '')
            ]) AS geo_id
            FROM award a
            WHERE a.recipient_entity_id = $1
               OR a.recipient_entity_id IN (
                   SELECT to_entity_id FROM relationship_edge WHERE from_entity_id = $1
                   UNION
                   SELECT from_entity_id FROM relationship_edge WHERE to_entity_id = $1
               )
        ) sub
        WHERE geo_id IS NOT NULL AND geo_id != ''
        "#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| r.get("geo_id")).collect())
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

pub async fn upsert_award(pool: &PgPool, req: &CreateAwardRequest) -> Result<Award> {
    let row = sqlx::query(
        r#"
        INSERT INTO award (generated_unique_award_id, recipient_entity_id, awarding_agency_entity_id,
                          funding_agency_entity_id, award_type, description, period_start, period_end,
                          obligation_amount, outlay_amount, place_of_performance, recipient_geo_id,
                          place_geo_id, evidence_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (generated_unique_award_id) DO UPDATE SET
            obligation_amount = EXCLUDED.obligation_amount,
            outlay_amount = EXCLUDED.outlay_amount,
            place_of_performance = COALESCE(EXCLUDED.place_of_performance, award.place_of_performance),
            recipient_geo_id = COALESCE(EXCLUDED.recipient_geo_id, award.recipient_geo_id),
            place_geo_id = COALESCE(EXCLUDED.place_geo_id, award.place_geo_id),
            description = COALESCE(EXCLUDED.description, award.description)
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

pub async fn upsert_lobbying_filing(
    pool: &PgPool,
    req: &CreateLobbyingFilingRequest,
) -> Result<LobbyingFiling> {
    let row = sqlx::query(
        r#"
        INSERT INTO lobbying_filing (filing_uuid, registrant_entity_id, client_entity_id, filing_type,
                                     filing_period, filing_year, income_or_expense, amount, issues_json, evidence_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (filing_uuid) DO UPDATE SET
            amount = COALESCE(EXCLUDED.amount, lobbying_filing.amount),
            issues_json = COALESCE(EXCLUDED.issues_json, lobbying_filing.issues_json)
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

pub async fn get_metric_history(
    pool: &PgPool,
    geo_id: &str,
    metric_id: &str,
    from_date: Option<chrono::NaiveDate>,
    to_date: Option<chrono::NaiveDate>,
    limit: i64,
) -> Result<Vec<MetricObservationWithDetails>> {
    let from = from_date.unwrap_or_else(|| {
        chrono::NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid default from date")
    });
    let to = to_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rows = sqlx::query(
        r#"
        SELECT mo.metric_id, mo.geo_id, mo.date, mo.value, mo.vintage_date,
               mo.release_date, mo.source_series_id, mo.evidence_id, mo.notes AS observation_notes,
               m.name AS metric_name, m.category, m.unit,
               m.seasonal_adjustment, m.source_id, m.notes AS metric_notes,
               ds.agency, ds.dataset, ds.url,
               ds.license, ds.update_frequency, ds.requires_api_key, ds.notes AS source_notes
        FROM metric_observation mo
        JOIN metric m ON mo.metric_id = m.metric_id
        JOIN data_source ds ON m.source_id = ds.source_id
        WHERE mo.geo_id = $1 AND mo.metric_id = $2
          AND mo.date >= $3 AND mo.date <= $4
        ORDER BY mo.date ASC, mo.vintage_date DESC
        LIMIT $5
        "#,
    )
    .bind(geo_id)
    .bind(metric_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(row_to_metric_observation_with_details)
        .collect())
}

pub async fn get_derived_metric_history(
    pool: &PgPool,
    geo_id: &str,
    metric_id: &str,
    limit: i64,
) -> Result<Vec<DerivedMetricObservation>> {
    let rows = sqlx::query(
        r#"
        SELECT metric_id, geo_id, date, value, formula, input_metric_ids, vintage_date, notes
        FROM derived_metric_observation
        WHERE geo_id = $1 AND metric_id = $2
        ORDER BY date ASC, vintage_date DESC
        LIMIT $3
        "#,
    )
    .bind(geo_id)
    .bind(metric_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(row_to_derived_metric_observation).collect())
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

pub async fn get_entity_economic_context(
    pool: &PgPool,
    entity_id: Uuid,
) -> Result<Vec<StateProfile>> {
    let geo_ids = get_linked_geo_ids(pool, entity_id).await?;
    let mut profiles = Vec::with_capacity(geo_ids.len());
    for geo_id in geo_ids {
        if let Some(profile) = get_state_profile(pool, &geo_id).await? {
            profiles.push(profile);
        }
    }
    Ok(profiles)
}

pub async fn unified_search(
    pool: &PgPool,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<UnifiedSearchResult> {
    let geo_limit = (limit / 2).min(10);
    let entity_limit = limit;

    let entities_result = search_entities(pool, query, entity_limit, offset).await?;
    let geos_result = search_geos(pool, query, geo_limit).await?;

    Ok(UnifiedSearchResult {
        entities: entities_result.entities,
        entity_total: entities_result.total,
        geos: geos_result.geos,
        geo_total: geos_result.total,
    })
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

pub async fn search_contributions(
    pool: &PgPool,
    query: &str,
    cycle: Option<i32>,
    limit: i64,
) -> Result<Vec<ContributionSearchResult>> {
    let pattern = format!("%{}%", query);

    let cycle_clause = match cycle {
        Some(c) => format!("AND cft.cycle = {}", c),
        None => String::new(),
    };

    let sql = format!(
        r#"
        SELECT
            cft.contributor_entity_id,
            e.display_name as contributor_name,
            COALESCE(SUM(cft.amount), 0.0) as total_amount,
            COUNT(cft.id) as transaction_count,
            (
                SELECT json_agg(json_build_object(
                    'recipient_entity_id', rc.id,
                    'recipient_name', COALESCE(rc.display_name, cft2.recipient_committee_name, cft2.committee_fec_id),
                    'total_amount', sub.total,
                    'transaction_count', sub.cnt
                ))
                FROM (
                    SELECT cft2.committee_entity_id, cft2.recipient_committee_name, cft2.committee_fec_id,
                           SUM(cft2.amount) as total, COUNT(*) as cnt
                    FROM campaign_finance_transaction cft2
                    WHERE cft2.contributor_entity_id = e.id {}
                    GROUP BY cft2.committee_entity_id, cft2.recipient_committee_name, cft2.committee_fec_id
                    ORDER BY total DESC
                    LIMIT 10
                ) sub
                LEFT JOIN entity rc ON rc.id = sub.committee_entity_id
            ) as top_recipients_json
        FROM entity e
        JOIN campaign_finance_transaction cft ON cft.contributor_entity_id = e.id
        WHERE e.entity_type = 'person'
            AND e.display_name ILIKE $1
        {}
        GROUP BY e.id, e.display_name
        ORDER BY total_amount DESC
        LIMIT $2
        "#,
        cycle_clause, cycle_clause
    );

    let rows = sqlx::query(&sql)
        .bind(&pattern)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    let results = rows
        .iter()
        .map(|r| {
            let top_json: Option<serde_json::Value> = r.get("top_recipients_json");
            let by_recipient = top_json
                .as_ref()
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|item| ContributionRecipientSummary {
                            recipient_entity_id: item
                                .get("recipient_entity_id")
                                .and_then(|v| v.as_str())
                                .and_then(|s| Uuid::parse_str(s).ok()),
                            recipient_name: item
                                .get("recipient_name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            total_amount: item
                                .get("total_amount")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                            transaction_count: item
                                .get("transaction_count")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        })
                        .collect()
                })
                .unwrap_or_default();

            ContributionSearchResult {
                contributor_entity_id: r.get("contributor_entity_id"),
                contributor_name: r.get("contributor_name"),
                total_amount: r.get("total_amount"),
                transaction_count: r.get("transaction_count"),
                top_recipients: by_recipient,
            }
        })
        .collect();

    Ok(results)
}

pub async fn get_entity_contribution_summary(
    pool: &PgPool,
    entity_id: Uuid,
) -> Result<Option<ContributionSummary>> {
    let row = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(amount), 0.0) as total_amount,
            COUNT(*) as transaction_count
        FROM campaign_finance_transaction
        WHERE contributor_entity_id = $1
        "#,
    )
    .bind(entity_id)
    .fetch_optional(pool)
    .await?;

    let given = match row {
        Some(r) => {
            let total: f64 = r.get("total_amount");
            let count: i64 = r.get("transaction_count");
            (total, count)
        }
        None => return Ok(None),
    };

    // Received (as committee)
    let recv_row = sqlx::query(
        r#"SELECT COALESCE(SUM(amount), 0.0) as total, COUNT(*) as cnt
           FROM campaign_finance_transaction WHERE committee_entity_id = $1"#,
    )
    .bind(entity_id)
    .fetch_one(pool)
    .await?;
    let _recv_total: f64 = recv_row.get("total");
    let _recv_count: i64 = recv_row.get("cnt");

    // Top recipients
    let recipient_rows = sqlx::query(
        r#"SELECT cft.committee_entity_id, COALESCE(e.display_name, cft.recipient_committee_name, cft.committee_fec_id) as name,
                  SUM(cft.amount) as total, COUNT(*) as cnt
           FROM campaign_finance_transaction cft
           LEFT JOIN entity e ON e.id = cft.committee_entity_id
           WHERE cft.contributor_entity_id = $1
           GROUP BY cft.committee_entity_id, name
           ORDER BY total DESC LIMIT 15"#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    let by_recipient: Vec<ContributionRecipientSummary> = recipient_rows
        .iter()
        .map(|r| ContributionRecipientSummary {
            recipient_entity_id: r.get("committee_entity_id"),
            recipient_name: r.get("name"),
            total_amount: r.get("total"),
            transaction_count: r.get("cnt"),
        })
        .collect();

    // By cycle
    let cycle_rows = sqlx::query(
        r#"SELECT cycle, SUM(amount) as total, COUNT(*) as cnt
           FROM campaign_finance_transaction
           WHERE contributor_entity_id = $1 AND cycle IS NOT NULL
           GROUP BY cycle ORDER BY cycle"#,
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    let by_cycle: Vec<ContributionCycleSummary> = cycle_rows
        .iter()
        .map(|r| ContributionCycleSummary {
            cycle: r.get("cycle"),
            total_amount: r.get("total"),
            transaction_count: r.get("cnt"),
        })
        .collect();

    Ok(Some(ContributionSummary {
        total_amount: given.0,
        transaction_count: given.1,
        by_recipient,
        by_cycle,
    }))
}

pub async fn get_entity_transactions(
    pool: &PgPool,
    entity_id: Uuid,
    cycle: Option<i32>,
    limit: i64,
) -> Result<Vec<CampaignFinanceTransaction>> {
    let cycle_clause = match cycle {
        Some(c) => format!("AND cft.cycle = {}", c),
        None => String::new(),
    };

    let sql = format!(
        r#"SELECT cft.id, cft.transaction_id, cft.committee_entity_id, cft.contributor_entity_id,
                  cft.candidate_entity_id, cft.amount, cft.date, cft.employer_text, cft.occupation_text,
                  cft.transaction_type, cft.evidence_id, cft.contributor_city, cft.contributor_state,
                  cft.contributor_zip, cft.committee_fec_id, cft.candidate_fec_id, cft.memo_text,
                  cft.transaction_type_code, cft.cycle, cft.recipient_committee_name,
                  cft.other_entity_id, cft.sub_id
           FROM campaign_finance_transaction cft
           WHERE cft.contributor_entity_id = $1 OR cft.committee_entity_id = $1
           {}
           ORDER BY cft.date DESC NULLS LAST, cft.amount DESC
           LIMIT $2"#,
        cycle_clause
    );

    let rows = sqlx::query(&sql)
        .bind(entity_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    rows.iter()
        .map(|r| {
            Ok(CampaignFinanceTransaction {
                id: r.get("id"),
                transaction_id: r.get("transaction_id"),
                committee_entity_id: r.get("committee_entity_id"),
                contributor_entity_id: r.get("contributor_entity_id"),
                candidate_entity_id: r.get("candidate_entity_id"),
                amount: r.get("amount"),
                date: r.get("date"),
                employer_text: r.get("employer_text"),
                occupation_text: r.get("occupation_text"),
                transaction_type: r.get("transaction_type"),
                evidence_id: r.get("evidence_id"),
                contributor_city: r.get("contributor_city"),
                contributor_state: r.get("contributor_state"),
                contributor_zip: r.get("contributor_zip"),
                committee_fec_id: r.get("committee_fec_id"),
                candidate_fec_id: r.get("candidate_fec_id"),
                memo_text: r.get("memo_text"),
                transaction_type_code: r.get("transaction_type_code"),
                cycle: r.get("cycle"),
                recipient_committee_name: r.get("recipient_committee_name"),
                other_entity_id: r.get("other_entity_id"),
                sub_id: r.get("sub_id"),
            })
        })
        .collect()
}

pub async fn search_candidates(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<CandidateSearchResult>> {
    let pattern = format!("%{}%", query);

    let rows = sqlx::query(
        r#"
        SELECT e.id as entity_id, e.display_name, cm.candidate_fec_id,
               cm.party, cm.office, cm.office_state, cm.election_year
        FROM entity e
        JOIN candidate_metadata cm ON cm.entity_id = e.id
        WHERE e.entity_type = 'candidate'
            AND (e.display_name ILIKE $1 OR cm.candidate_fec_id ILIKE $1)
        ORDER BY
            CASE
                WHEN e.display_name ILIKE $1 THEN similarity(e.display_name, $2)
                ELSE 0
            END DESC,
            e.display_name
        LIMIT $3
        "#,
    )
    .bind(&pattern)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| CandidateSearchResult {
            entity_id: r.get("entity_id"),
            display_name: r.get("display_name"),
            candidate_fec_id: r.get("candidate_fec_id"),
            party: r.get("party"),
            office: r.get("office"),
            office_state: r.get("office_state"),
            election_year: r.get("election_year"),
        })
        .collect())
}

pub async fn get_candidate_info(pool: &PgPool, entity_id: Uuid) -> Result<Option<CandidateInfo>> {
    let row = sqlx::query(
        r#"SELECT entity_id, candidate_fec_id, party, office, office_state, office_district,
                  incumbent_challenge, election_year, candidate_status, principal_committee_fec_id
           FROM candidate_metadata WHERE entity_id = $1"#,
    )
    .bind(entity_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| CandidateInfo {
        entity_id: r.get("entity_id"),
        candidate_fec_id: r.get("candidate_fec_id"),
        party: r.get("party"),
        office: r.get("office"),
        office_state: r.get("office_state"),
        office_district: r.get("office_district"),
        incumbent_challenge: r.get("incumbent_challenge"),
        election_year: r.get("election_year"),
        candidate_status: r.get("candidate_status"),
        principal_committee_fec_id: r.get("principal_committee_fec_id"),
    }))
}

pub async fn get_committee_info(pool: &PgPool, entity_id: Uuid) -> Result<Option<CommitteeInfo>> {
    let row = sqlx::query(
        r#"SELECT entity_id, committee_fec_id, committee_type, committee_designation,
                  party, treasurer_name, organization_type, connected_organization
           FROM committee_metadata WHERE entity_id = $1"#,
    )
    .bind(entity_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| CommitteeInfo {
        entity_id: r.get("entity_id"),
        committee_fec_id: r.get("committee_fec_id"),
        committee_type: r.get("committee_type"),
        committee_designation: r.get("committee_designation"),
        party: r.get("party"),
        treasurer_name: r.get("treasurer_name"),
        organization_type: r.get("organization_type"),
        connected_organization: r.get("connected_organization"),
    }))
}

pub async fn get_money_flow_to_candidate(
    pool: &PgPool,
    from_entity_id: Uuid,
    to_candidate_id: Uuid,
) -> Result<MoneyFlowResult> {
    // Find committees that support this candidate
    let committee_rows = sqlx::query(
        r#"SELECT ccl.committee_entity_id, ccl.committee_fec_id, COALESCE(e.display_name, ccl.committee_fec_id) as name
           FROM candidate_committee_link ccl
           LEFT JOIN entity e ON e.id = ccl.committee_entity_id
           WHERE ccl.candidate_entity_id = $1"#,
    )
    .bind(to_candidate_id)
    .fetch_all(pool)
    .await?;

    let _candidate_name = sqlx::query("SELECT display_name FROM entity WHERE id = $1")
        .bind(to_candidate_id)
        .fetch_optional(pool)
        .await?
        .map(|r| r.get::<String, _>("display_name"))
        .unwrap_or_default();

    let mut flows = Vec::new();
    let mut total = 0.0f64;

    for row in &committee_rows {
        let cmte_entity_id: Option<Uuid> = row.get("committee_entity_id");
        let cmte_fec_id: String = row.get("committee_fec_id");
        let cmte_name: String = row.get("name");

        if let Some(cmte_id) = cmte_entity_id {
            let amt_row = sqlx::query(
                r#"SELECT COALESCE(SUM(amount), 0.0) as total, COUNT(*) as cnt
                   FROM campaign_finance_transaction
                   WHERE contributor_entity_id = $1 AND committee_entity_id = $2"#,
            )
            .bind(from_entity_id)
            .bind(cmte_id)
            .fetch_one(pool)
            .await?;

            let amount: f64 = amt_row.get("total");
            let count: i64 = amt_row.get("cnt");

            if amount > 0.0 || count > 0 {
                flows.push(MoneyFlowStep {
                    committee_entity_id: Some(cmte_id),
                    committee_name: cmte_name,
                    committee_fec_id: cmte_fec_id,
                    amount,
                    transaction_count: count,
                });
                total += amount;
            }
        }
    }

    // Also check direct committee-to-candidate contributions
    flows.sort_by(|a, b| {
        b.amount
            .partial_cmp(&a.amount)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(MoneyFlowResult {
        from_entity_id,
        to_candidate_id,
        total_amount: total,
        flows,
    })
}

pub async fn get_top_donors_to_committee(
    pool: &PgPool,
    cmte_fec_id: &str,
    limit: i64,
) -> Result<Vec<ContributionRecipientSummary>> {
    let rows = sqlx::query(
        r#"
        SELECT cft.contributor_entity_id, COALESCE(e.display_name, cft.recipient_committee_name) as name,
               SUM(cft.amount) as total, COUNT(*) as cnt
        FROM campaign_finance_transaction cft
        LEFT JOIN entity e ON e.id = cft.contributor_entity_id
        WHERE cft.committee_fec_id = $1
        GROUP BY cft.contributor_entity_id, name
        ORDER BY total DESC
        LIMIT $2
        "#,
    )
    .bind(cmte_fec_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| ContributionRecipientSummary {
            recipient_entity_id: r.get("contributor_entity_id"),
            recipient_name: r.get("name"),
            total_amount: r.get("total"),
            transaction_count: r.get("cnt"),
        })
        .collect())
}
