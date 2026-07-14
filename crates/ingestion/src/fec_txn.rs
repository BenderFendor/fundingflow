use std::collections::{HashMap, HashSet};

use anyhow::Result;
use memmap2::Mmap;
use sqlx::PgPool;
use sqlx::Row;
use tracing::{info, warn};
use uuid::Uuid;

use crate::ImportSummary;
use crate::normalize_name;

const TXN_BATCH: usize = 10_000;

#[derive(Debug, Clone)]
pub struct FecTransaction {
    pub cmte_id: String,
    pub entity_tp: String,
    pub name: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub employer: String,
    pub occupation: String,
    pub date: Option<chrono::NaiveDate>,
    pub amount: f64,
    pub other_id: String,
    pub cand_id: String,
    pub sub_id: String,
    pub memo_text: String,
    pub line_num: String,
}

pub struct EntityCache {
    pub persons: HashMap<String, Uuid>,
    pub committees: HashMap<String, Uuid>,
    pub existing_person_names: HashSet<String>,
}

impl EntityCache {
    pub fn new(committees: HashMap<String, Uuid>) -> Self {
        Self {
            persons: HashMap::new(),
            committees,
            existing_person_names: HashSet::new(),
        }
    }
}

pub struct ProcessParams<'a> {
    pub txn_type: &'a str,
    pub edge_type: &'a str,
    pub cycle: i32,
    pub evidence_id: Uuid,
}

fn parse_fec_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%m%d%Y")
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .ok()
}

pub fn parse_line(line: &[u8]) -> Option<FecTransaction> {
    let fields: Vec<&[u8]> = line.split(|&b| b == b'|').collect();
    if fields.len() < 15 {
        return None;
    }

    let s = |i: usize| -> String {
        fields
            .get(i)
            .and_then(|f| str::from_utf8(f).ok())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    let cmte_id = s(0);
    let name = s(7);
    if cmte_id.is_empty() || name.is_empty() {
        return None;
    }

    let amount: f64 = s(14).parse().unwrap_or(0.0);
    let sub_id = s(fields.len() - 1);

    let raw_cand_id = s(16);
    let cand_id =
        if raw_cand_id.len() >= 3 && matches!(raw_cand_id.as_bytes()[0], b'H' | b'S' | b'P') {
            raw_cand_id
        } else {
            String::new()
        };

    Some(FecTransaction {
        cmte_id,
        entity_tp: s(6),
        name,
        city: s(8),
        state: s(9),
        zip: s(10),
        employer: s(11),
        occupation: s(12),
        date: parse_fec_date(&s(13)),
        amount,
        other_id: s(15),
        cand_id,
        sub_id,
        memo_text: if fields.len() > 19 {
            s(19)
        } else {
            String::new()
        },
        line_num: s(5),
    })
}

pub async fn process_transaction_file(
    pool: &PgPool,
    mmap: &Mmap,
    params: &ProcessParams<'_>,
    cache: &mut EntityCache,
    summary: &mut ImportSummary,
) -> Result<()> {
    load_existing_person_names(pool, &mut cache.existing_person_names).await;

    let mut batch: Vec<FecTransaction> = Vec::with_capacity(TXN_BATCH);
    let mut pos = 0usize;

    if let Some(p) = mmap.iter().position(|&b| b == b'\n') {
        pos = p + 1;
    }

    let mut total_count = 0u64;

    while pos < mmap.len() {
        let end = mmap[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|e| pos + e)
            .unwrap_or(mmap.len());

        if end > pos + 20
            && let Some(txn) = parse_line(&mmap[pos..end])
        {
            batch.push(txn);
        }

        pos = if end < mmap.len() { end + 1 } else { end };

        if batch.len() >= TXN_BATCH {
            let count = batch.len();
            process_batch(pool, &batch, params, cache, summary).await;
            total_count += count as u64;
            batch.clear();

            if total_count.is_multiple_of(100_000) {
                info!(
                    "FEC {}: processed {} transactions",
                    params.txn_type, total_count
                );
            }
        }
    }

    if !batch.is_empty() {
        total_count += batch.len() as u64;
        process_batch(pool, &batch, params, cache, summary).await;
    }

    info!(
        "FEC {}: total {} transactions processed",
        params.txn_type, total_count
    );
    Ok(())
}

async fn process_batch(
    pool: &PgPool,
    batch: &[FecTransaction],
    params: &ProcessParams<'_>,
    cache: &mut EntityCache,
    summary: &mut ImportSummary,
) {
    if params.txn_type == "individual" {
        ensure_person_entities(pool, batch, cache, summary).await;
    } else {
        ensure_donor_committees(pool, batch, cache, summary).await;
    }

    let inserted = batch_insert_transactions(pool, batch, params, cache).await;
    summary.records_imported += inserted;

    batch_create_edges(pool, batch, params, cache).await;
}

async fn ensure_person_entities(
    pool: &PgPool,
    batch: &[FecTransaction],
    cache: &mut EntityCache,
    summary: &mut ImportSummary,
) {
    let mut new_seen: HashSet<String> = HashSet::new();
    let mut new_names: Vec<String> = Vec::new();
    let mut new_displays: Vec<String> = Vec::new();

    for txn in batch {
        let canonical = normalize_name(&txn.name);
        if cache.persons.contains_key(&canonical)
            || cache.existing_person_names.contains(&canonical)
        {
            continue;
        }
        if new_seen.insert(canonical.clone()) {
            cache.existing_person_names.insert(canonical.clone());
            new_displays.push(txn.name.clone());
            new_names.push(canonical);
        }
    }

    if new_names.is_empty() {
        return;
    }

    let mut created = 0u64;
    for chunk in new_displays.chunks(5000) {
        let end = chunk.len().min(new_names.len());
        let names_slice = &new_names[..end];
        let name_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
        let can_refs: Vec<&str> = names_slice.iter().map(|s| s.as_str()).collect();

        match sqlx::query(
            r#"INSERT INTO entity (entity_type, display_name, canonical_name)
               SELECT 'person'::entity_type, t.name, t.canonical
               FROM unnest($1::text[], $2::text[]) AS t(name, canonical)
               ON CONFLICT DO NOTHING
               RETURNING id, canonical_name"#,
        )
        .bind(&name_refs)
        .bind(&can_refs)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => {
                for row in &rows {
                    let id: Uuid = row.get("id");
                    let can: String = row.get("canonical_name");
                    cache.persons.insert(can, id);
                }
                created += rows.len() as u64;
            }
            Err(e) => {
                warn!("FEC: batch person insert failed: {e}");
            }
        }
    }
    summary.entities_created += created;

    // Reconcile any names that exist in DB but weren't in cache
    let missing: Vec<String> = batch
        .iter()
        .map(|t| normalize_name(&t.name))
        .filter(|n| !cache.persons.contains_key(n))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    if !missing.is_empty() {
        let found = lookup_persons_by_name(pool, &missing).await;
        for (name, id) in found {
            cache.persons.insert(name.clone(), id);
            cache.existing_person_names.insert(name);
        }
    }
}

async fn lookup_persons_by_name(pool: &PgPool, names: &[String]) -> Vec<(String, Uuid)> {
    if names.is_empty() {
        return vec![];
    }
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    match sqlx::query(
        "SELECT id, canonical_name FROM entity WHERE entity_type = 'person' AND canonical_name = ANY($1::text[])",
    )
    .bind(&name_refs)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .iter()
            .map(|r| (r.get("canonical_name"), r.get("id")))
            .collect(),
        Err(_) => vec![],
    }
}

async fn load_existing_person_names(pool: &PgPool, set: &mut HashSet<String>) {
    let rows = sqlx::query("SELECT canonical_name FROM entity WHERE entity_type = 'person'")
        .fetch_all(pool)
        .await;
    if let Ok(rows) = rows {
        for row in rows {
            let name: String = row.get("canonical_name");
            set.insert(name);
        }
    }
}

async fn ensure_donor_committees(
    pool: &PgPool,
    batch: &[FecTransaction],
    cache: &mut EntityCache,
    summary: &mut ImportSummary,
) {
    let new_cmte_ids: Vec<&str> = batch
        .iter()
        .filter_map(|t| {
            if t.other_id.is_empty() || cache.committees.contains_key(&t.other_id) {
                None
            } else {
                Some(t.other_id.as_str())
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if new_cmte_ids.is_empty() {
        return;
    }

    let mut created = 0u64;
    for chunk in new_cmte_ids.chunks(5000) {
        let names: Vec<String> = chunk
            .iter()
            .map(|id| format!("FEC Committee {}", id))
            .collect();
        let canonicals: Vec<String> = chunk
            .iter()
            .map(|id| normalize_name(&format!("fec committee {}", id)))
            .collect();
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let can_refs: Vec<&str> = canonicals.iter().map(|s| s.as_str()).collect();

        let rows = match sqlx::query(
            r#"INSERT INTO entity (entity_type, display_name, canonical_name)
               SELECT 'committee'::entity_type, t.name, t.canonical
               FROM unnest($1::text[], $2::text[]) AS t(name, canonical)
               ON CONFLICT DO NOTHING
               RETURNING id"#,
        )
        .bind(&name_refs)
        .bind(&can_refs)
        .fetch_all(pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("FEC: ensure committee insert failed: {e}");
                continue;
            }
        };

        let entity_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

        if let Err(e) = sqlx::query(
            r#"INSERT INTO entity_identifier (entity_id, source, identifier_type, identifier_value)
               SELECT unnest($1::uuid[]), 'fec'::identifier_source, 'fec_committee_id'::identifier_type, unnest($2::text[])
               ON CONFLICT DO NOTHING"#,
        )
        .bind(&entity_ids)
        .bind(chunk)
        .execute(pool)
        .await
        {
            warn!("FEC: ensure committee identifier failed: {e}");
        }

        for (i, &cmte_id) in chunk.iter().enumerate() {
            if i < entity_ids.len() {
                cache.committees.insert(cmte_id.to_string(), entity_ids[i]);
            }
        }
        created += entity_ids.len() as u64;
    }
    summary.entities_created += created;
}

async fn batch_insert_transactions(
    pool: &PgPool,
    batch: &[FecTransaction],
    params: &ProcessParams<'_>,
    cache: &EntityCache,
) -> u64 {
    let mut valid: Vec<(&FecTransaction, Uuid, Option<Uuid>)> = Vec::with_capacity(batch.len());

    for txn in batch {
        let cmte_entity_id = match cache.committees.get(&txn.cmte_id) {
            Some(id) => *id,
            None => continue,
        };

        let contributor_eid = if params.txn_type == "individual" {
            let canonical = normalize_name(&txn.name);
            cache.persons.get(&canonical).copied()
        } else if !txn.other_id.is_empty() {
            cache.committees.get(&txn.other_id).copied()
        } else {
            None
        };

        valid.push((txn, cmte_entity_id, contributor_eid));
    }

    if valid.is_empty() {
        return 0;
    }

    let mut inserted = 0u64;

    for chunk in valid.chunks(TXN_BATCH) {
        let sub_ids: Vec<String> = chunk.iter().map(|(t, _, _)| t.sub_id.clone()).collect();
        let cmte_eids: Vec<Uuid> = chunk.iter().map(|(_, eid, _)| *eid).collect();
        let contributor_eids: Vec<Option<Uuid>> = chunk.iter().map(|(_, _, c)| *c).collect();
        let cmte_fec_ids: Vec<String> = chunk.iter().map(|(t, _, _)| t.cmte_id.clone()).collect();
        let amounts: Vec<f64> = chunk.iter().map(|(t, _, _)| t.amount).collect();
        let dates: Vec<Option<chrono::NaiveDate>> = chunk.iter().map(|(t, _, _)| t.date).collect();
        let employers: Vec<String> = chunk.iter().map(|(t, _, _)| t.employer.clone()).collect();
        let occupations: Vec<String> = chunk.iter().map(|(t, _, _)| t.occupation.clone()).collect();
        let cities: Vec<String> = chunk.iter().map(|(t, _, _)| t.city.clone()).collect();
        let states: Vec<String> = chunk.iter().map(|(t, _, _)| t.state.clone()).collect();
        let zips: Vec<String> = chunk.iter().map(|(t, _, _)| t.zip.clone()).collect();
        let memos: Vec<String> = chunk.iter().map(|(t, _, _)| t.memo_text.clone()).collect();
        let line_nums: Vec<String> = chunk.iter().map(|(t, _, _)| t.line_num.clone()).collect();
        let cand_ids: Vec<String> = chunk.iter().map(|(t, _, _)| t.cand_id.clone()).collect();
        let recipient_names: Vec<String> = chunk
            .iter()
            .map(|(t, _, _)| format!("FEC Committee {}", t.cmte_id))
            .collect();

        let txn_types: Vec<&str> = vec![params.txn_type; chunk.len()];
        let cycles: Vec<i32> = vec![params.cycle; chunk.len()];
        let evidence_ids: Vec<Uuid> = vec![params.evidence_id; chunk.len()];

        let result = sqlx::query(
            r#"INSERT INTO campaign_finance_transaction
               (transaction_id, sub_id, committee_entity_id, contributor_entity_id,
                amount, date, employer_text, occupation_text, transaction_type,
                contributor_city, contributor_state, contributor_zip,
                committee_fec_id, candidate_fec_id, memo_text, transaction_type_code,
                cycle, recipient_committee_name, evidence_id)
               SELECT
                 unnest($1::text[]), unnest($2::text[]), unnest($3::uuid[]), unnest($4::uuid[]),
                 unnest($5::float8[]), unnest($6::date[]), unnest($7::text[]), unnest($8::text[]),
                 unnest($9::text[]), unnest($10::text[]), unnest($11::text[]), unnest($12::text[]),
                 unnest($13::text[]), unnest($14::text[]), unnest($15::text[]), unnest($16::text[]),
                 unnest($17::int[]), unnest($18::text[]), unnest($19::uuid[])
               ON CONFLICT (sub_id) DO NOTHING"#,
        )
        .bind(&sub_ids)
        .bind(&sub_ids)
        .bind(&cmte_eids)
        .bind(&contributor_eids)
        .bind(&amounts)
        .bind(&dates)
        .bind(&employers)
        .bind(&occupations)
        .bind(&txn_types)
        .bind(&cities)
        .bind(&states)
        .bind(&zips)
        .bind(&cmte_fec_ids)
        .bind(&cand_ids)
        .bind(&memos)
        .bind(&line_nums)
        .bind(&cycles)
        .bind(&recipient_names)
        .bind(&evidence_ids)
        .execute(pool)
        .await;

        match result {
            Ok(r) => inserted += r.rows_affected(),
            Err(e) => warn!("FEC: batch txn insert failed: {e}"),
        }
    }

    inserted
}

async fn batch_create_edges(
    pool: &PgPool,
    batch: &[FecTransaction],
    params: &ProcessParams<'_>,
    cache: &EntityCache,
) {
    let mut valid: Vec<(Uuid, Uuid, f64, Option<chrono::NaiveDate>)> = Vec::new();

    for txn in batch {
        let (from_id, to_id) = if params.edge_type == "person_contributed_to_committee" {
            let canonical = normalize_name(&txn.name);
            match (
                cache.persons.get(&canonical),
                cache.committees.get(&txn.cmte_id),
            ) {
                (Some(from), Some(to)) => (*from, *to),
                _ => continue,
            }
        } else {
            let from = if txn.other_id.is_empty() {
                None
            } else {
                cache.committees.get(&txn.other_id)
            };
            match (from, cache.committees.get(&txn.cmte_id)) {
                (Some(from), Some(to)) => (*from, *to),
                _ => continue,
            }
        };

        valid.push((from_id, to_id, txn.amount, txn.date));
    }

    for chunk in valid.chunks(TXN_BATCH) {
        let from_ids: Vec<Uuid> = chunk.iter().map(|(f, _, _, _)| *f).collect();
        let to_ids: Vec<Uuid> = chunk.iter().map(|(_, t, _, _)| *t).collect();
        let amounts: Vec<f64> = chunk.iter().map(|(_, _, a, _)| *a).collect();
        let dates: Vec<Option<chrono::NaiveDate>> = chunk.iter().map(|(_, _, _, d)| *d).collect();
        let currencies: Vec<&str> = vec!["USD"; chunk.len()];
        let confidences: Vec<f64> = vec![1.0; chunk.len()];
        let evidence_ids: Vec<Uuid> = vec![params.evidence_id; chunk.len()];
        let sources: Vec<&str> = vec!["fec"; chunk.len()];
        let edge_types: Vec<&str> = vec![params.edge_type; chunk.len()];

        if let Err(e) = sqlx::query(
            r#"INSERT INTO relationship_edge
               (from_entity_id, to_entity_id, edge_type, amount, currency, confidence, evidence_id, source, started_on)
               SELECT
                 unnest($1::uuid[]), unnest($2::uuid[]), unnest($3::text[])::edge_type,
                 unnest($4::float8[]), unnest($5::text[]), unnest($6::float8[]),
                 unnest($7::uuid[]), unnest($8::text[]), unnest($9::date[])"#,
        )
        .bind(&from_ids)
        .bind(&to_ids)
        .bind(&edge_types)
        .bind(&amounts)
        .bind(&currencies)
        .bind(&confidences)
        .bind(&evidence_ids)
        .bind(&sources)
        .bind(&dates)
        .execute(pool)
        .await
        {
            warn!("FEC: batch edge insert failed: {e}");
        }
    }
}
