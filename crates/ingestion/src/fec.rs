use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::Result;
use async_trait::async_trait;
use db::queries;
use memmap2::Mmap;
use sqlx::PgPool;
use sqlx::Row;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};
use uuid::Uuid;
use zip::ZipArchive;

use crate::fec_txn::{EntityCache, ProcessParams, process_transaction_file};
use crate::{ImportSummary, Importer, content_hash, normalize_name};

const ENTITY_BATCH: usize = 5000;

#[derive(Debug, Clone)]
struct CandidateEntry {
    cand_id: String,
    name: String,
    party: String,
    election_year: String,
    office: String,
    office_st: String,
    office_district: String,
    ici: String,
    status: String,
    pcc: String,
}

#[derive(Debug, Clone)]
struct CommitteeEntry {
    cmte_id: String,
    cmte_name: String,
    treasurer: String,
    designation: String,
    cmte_tp: String,
    party: String,
    org_tp: String,
    connected_org: String,
    #[allow(dead_code)]
    cand_id: String,
    #[allow(dead_code)]
    filing_freq: String,
}

#[derive(Debug, Clone)]
struct CclEntry {
    cand_id: String,
    cmte_id: String,
    cmte_designation: String,
}

pub struct FecImporter {
    pub base_url: String,
    pub client: reqwest::Client,
    pub year: i32,
}

impl FecImporter {
    pub fn new(base_url: impl Into<String>, year: i32) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .connect_timeout(std::time::Duration::from_secs(30))
                .user_agent("FundingFlow/0.1 (public-record research)")
                .build()
                .unwrap(),
            year,
        }
    }

    fn ys(&self) -> String {
        format!("{:02}", self.year % 100)
    }

    fn zip_url(&self, base: &str) -> String {
        format!("{}/{}/{}{}.zip", self.base_url, self.year, base, self.ys())
    }

    fn cache_path(&self, base: &str) -> PathBuf {
        PathBuf::from("data/fec").join(format!("{}{}.zip", base, self.ys()))
    }

    fn temp_path(&self, prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}{}.txt", prefix, self.ys()))
    }

    async fn download_to_temp(&self, base: &str) -> Result<PathBuf> {
        let cache = self.cache_path(base);
        if cache.exists() {
            let meta = tokio::fs::metadata(&cache).await?;
            info!(
                "FEC: using cached {} ({} bytes)",
                cache.display(),
                meta.len()
            );
            return Ok(cache);
        }
        let url = self.zip_url(base);
        info!("FEC: downloading {}", url);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("FEC download failed for {}: HTTP {}", url, resp.status());
        }
        if let Some(parent) = cache.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::File::create(&cache).await?;
        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            file.write_all(&chunk?).await?;
        }
        file.flush().await?;
        info!("FEC: saved {} to {:?}", url, cache);
        Ok(cache)
    }

    fn read_from_zip(path: &Path, entry: &str) -> Result<String> {
        let file = std::fs::File::open(path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entry_file = archive.by_name(entry)?;
        let mut content = String::new();
        entry_file.read_to_string(&mut content)?;
        Ok(content)
    }

    fn extract_entry_to_temp(zip_path: &Path, entry: &str, out_path: &Path) -> Result<()> {
        if out_path.exists() {
            info!("FEC: using cached {}", out_path.display());
            return Ok(());
        }
        info!("FEC: extracting {} from zip...", entry);
        let file = std::fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entry_file = archive.by_name(entry)?;
        let mut out = std::fs::File::create(out_path)?;
        std::io::copy(&mut entry_file, &mut out)?;
        info!("FEC: extracted to {}", out_path.display());
        Ok(())
    }

    fn parse_candidate_master(content: &str) -> Vec<CandidateEntry> {
        content
            .lines()
            .skip(1)
            .filter_map(|line| {
                let p: Vec<&str> = line.split('|').collect();
                if p.len() < 10 {
                    return None;
                }
                Some(CandidateEntry {
                    cand_id: p[0].trim().to_string(),
                    name: p[1].trim().to_string(),
                    party: p[2].trim().to_string(),
                    election_year: p[3].trim().to_string(),
                    office_st: p[4].trim().to_string(),
                    office: p[5].trim().to_string(),
                    office_district: p[6].trim().to_string(),
                    ici: p[7].trim().to_string(),
                    status: p[8].trim().to_string(),
                    pcc: p[9].trim().to_string(),
                })
            })
            .filter(|c| !c.cand_id.is_empty() && !c.name.is_empty())
            .collect()
    }

    fn parse_committee_master(content: &str) -> HashMap<String, CommitteeEntry> {
        let mut map = HashMap::new();
        for line in content.lines().skip(1) {
            let p: Vec<&str> = line.split('|').collect();
            if p.len() < 10 {
                continue;
            }
            let cmte_id = p[0].trim().to_string();
            if cmte_id.is_empty() {
                continue;
            }
            map.insert(
                cmte_id.clone(),
                CommitteeEntry {
                    cmte_id,
                    cmte_name: p[1].trim().to_string(),
                    treasurer: p[2].trim().to_string(),
                    designation: p.get(8).map(|s| s.trim().to_string()).unwrap_or_default(),
                    cmte_tp: p.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                    party: p.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                    filing_freq: p.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
                    org_tp: p.get(12).map(|s| s.trim().to_string()).unwrap_or_default(),
                    connected_org: p.get(13).map(|s| s.trim().to_string()).unwrap_or_default(),
                    cand_id: p.get(14).map(|s| s.trim().to_string()).unwrap_or_default(),
                },
            );
        }
        map
    }

    fn parse_ccl(content: &str) -> Vec<CclEntry> {
        content
            .lines()
            .skip(1)
            .filter_map(|line| {
                let p: Vec<&str> = line.split('|').collect();
                if p.len() < 5 {
                    return None;
                }
                Some(CclEntry {
                    cand_id: p[0].trim().to_string(),
                    cmte_id: p[3].trim().to_string(),
                    cmte_designation: p.get(5).map(|s| s.trim().to_string()).unwrap_or_default(),
                })
            })
            .filter(|c| !c.cand_id.is_empty() && !c.cmte_id.is_empty())
            .collect()
    }
}

#[async_trait]
impl Importer for FecImporter {
    fn source_name(&self) -> &'static str {
        "fec"
    }

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary> {
        let mut summary = ImportSummary {
            source: "fec".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };
        let ys = self.ys();

        // 1. Candidate master (cn)
        let cn_path = match self.download_to_temp("cn").await {
            Ok(p) => p,
            Err(e) => {
                warn!("FEC: could not download candidate master: {e}");
                return Ok(summary);
            }
        };
        let cn_content = Self::read_from_zip(&cn_path, "cn.txt")?;
        let candidates = Self::parse_candidate_master(&cn_content);
        info!("FEC: parsed {} candidates", candidates.len());

        let evidence = create_fec_evidence(pool, &ys, "candidate_master").await?;
        let cand_id_map =
            batch_create_candidates(pool, &candidates, evidence.id, &mut summary).await;
        info!("FEC: created {} candidate entities", cand_id_map.len());

        // 2. Committee master (cm)
        let cm_path = self.download_to_temp("cm").await?;
        let cm_content = Self::read_from_zip(&cm_path, "cm.txt")?;
        let committees = Self::parse_committee_master(&cm_content);
        info!("FEC: parsed {} committees", committees.len());

        let cm_evidence = create_fec_evidence(pool, &ys, "committee_master").await?;
        let cmte_id_map =
            batch_create_committees(pool, &committees, cm_evidence.id, &mut summary).await;
        info!("FEC: created {} committee entities", cmte_id_map.len());

        // 3. Candidate-committee linkage (ccl)
        let ccl_path = match self.download_to_temp("ccl").await {
            Ok(p) => p,
            Err(e) => {
                warn!("FEC: could not download ccl: {e}");
                return Ok(summary);
            }
        };
        let ccl_content = Self::read_from_zip(&ccl_path, "ccl.txt")?;
        let ccl_entries = Self::parse_ccl(&ccl_content);
        batch_create_ccl_links(pool, &ccl_entries, &cand_id_map, &cmte_id_map).await;
        info!(
            "FEC: created {} candidate-committee links",
            ccl_entries.len()
        );

        // 4. Individual contributions (indiv) - person -> committee
        let indiv_summary = self
            .process_txn_file(
                pool,
                "indiv",
                "itcont.txt",
                "individual",
                "person_contributed_to_committee",
                &cmte_id_map,
                evidence.id,
            )
            .await?;
        merge_summary(&mut summary, &indiv_summary);

        // 5. Committee contributions to candidates (pas2) - committee -> candidate/committee
        let pas2_summary = self
            .process_txn_file(
                pool,
                "pas2",
                "itpas2.txt",
                "committee_to_candidate",
                "committee_contributed_to_candidate",
                &cmte_id_map,
                evidence.id,
            )
            .await?;
        merge_summary(&mut summary, &pas2_summary);

        // 6. Transfers between committees (oth) - committee -> committee
        let oth_summary = self
            .process_txn_file(
                pool,
                "oth",
                "itoth.txt",
                "committee_to_committee",
                "committee_transferred_to_committee",
                &cmte_id_map,
                evidence.id,
            )
            .await?;
        merge_summary(&mut summary, &oth_summary);

        info!(
            "FEC import done: {} records, {} entities, {} errors",
            summary.records_imported, summary.entities_created, summary.errors
        );

        Ok(summary)
    }
}

impl FecImporter {
    #[allow(clippy::too_many_arguments)]
    async fn process_txn_file(
        &self,
        pool: &PgPool,
        file_prefix: &str,
        entry_name: &str,
        txn_type: &str,
        edge_type: &str,
        cmte_id_map: &HashMap<String, Uuid>,
        fec_evidence_id: Uuid,
    ) -> Result<ImportSummary> {
        let mut summary = ImportSummary {
            source: "fec".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };

        let zip_path = match self.download_to_temp(file_prefix).await {
            Ok(p) => p,
            Err(e) => {
                warn!("FEC: could not download {}: {e}", file_prefix);
                return Ok(summary);
            }
        };

        let tmp_path = self.temp_path(file_prefix);
        if let Err(e) = Self::extract_entry_to_temp(&zip_path, entry_name, &tmp_path) {
            warn!("FEC: could not extract {}: {e}", entry_name);
            return Ok(summary);
        }

        let file = match std::fs::File::open(&tmp_path) {
            Ok(f) => f,
            Err(e) => {
                warn!("FEC: could not open {}: {e}", tmp_path.display());
                return Ok(summary);
            }
        };

        let mmap = unsafe {
            match Mmap::map(&file) {
                Ok(m) => m,
                Err(e) => {
                    warn!("FEC: could not mmap: {e}");
                    return Ok(summary);
                }
            }
        };

        info!(
            "FEC: processing {} ({} bytes), type={}",
            file_prefix,
            mmap.len(),
            txn_type
        );

        let mut cache = EntityCache::new(cmte_id_map.clone());

        let params = ProcessParams {
            txn_type,
            edge_type,
            cycle: self.year,
            evidence_id: fec_evidence_id,
        };

        process_transaction_file(pool, &mmap, &params, &mut cache, &mut summary).await?;

        let _ = std::fs::remove_file(&tmp_path);

        info!(
            "FEC: {} done: {} records, {} entities, {} errors",
            file_prefix, summary.records_imported, summary.entities_created, summary.errors
        );

        Ok(summary)
    }
}

async fn create_fec_evidence(pool: &PgPool, ys: &str, record_type: &str) -> Result<Evidence> {
    let source_record = queries::upsert_source_record(
        pool,
        "fec",
        record_type,
        &format!("fec-{}-{}", record_type, ys),
        Some(&format!("https://www.fec.gov/files/bulk-downloads/{}/", ys)),
        &content_hash(&format!("{}-{}", record_type, ys)),
        None,
    )
    .await?;

    let evidence = queries::create_evidence(
        pool,
        source_record.id,
        None,
        Some(&format!("https://www.fec.gov/files/bulk-downloads/{}/", ys)),
        Some(1.0),
    )
    .await?;

    Ok(evidence)
}

use domain::Evidence;

async fn batch_create_candidates(
    pool: &PgPool,
    candidates: &[CandidateEntry],
    evidence_id: Uuid,
    summary: &mut ImportSummary,
) -> HashMap<String, Uuid> {
    let mut map = HashMap::new();

    // Load existing
    let existing = load_existing_fec_ids(pool, "fec_candidate_id").await;
    for (fec_id, entity_id) in &existing {
        map.insert(fec_id.clone(), *entity_id);
    }

    let new_candidates: Vec<&CandidateEntry> = candidates
        .iter()
        .filter(|c| !map.contains_key(&c.cand_id))
        .collect();

    if new_candidates.is_empty() {
        return map;
    }

    for chunk in new_candidates.chunks(ENTITY_BATCH) {
        let names: Vec<&str> = chunk.iter().map(|c| c.name.as_str()).collect();
        let canonicals: Vec<String> = chunk.iter().map(|c| normalize_name(&c.name)).collect();
        let can_refs: Vec<&str> = canonicals.iter().map(|s| s.as_str()).collect();
        let fec_ids: Vec<&str> = chunk.iter().map(|c| c.cand_id.as_str()).collect();

        let rows = match sqlx::query(
            r#"
            INSERT INTO entity (entity_type, display_name, canonical_name)
            SELECT 'candidate'::entity_type, t.name, t.canonical
            FROM unnest($1::text[], $2::text[]) AS t(name, canonical)
            RETURNING id, canonical_name
            "#,
        )
        .bind(&names)
        .bind(&can_refs)
        .fetch_all(pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("FEC: batch candidate insert failed: {e}");
                summary.errors += chunk.len() as u64;
                continue;
            }
        };

        let entity_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

        // Insert identifiers
        if let Err(e) = sqlx::query(
            r#"INSERT INTO entity_identifier (entity_id, source, identifier_type, identifier_value, evidence_id)
               SELECT unnest($1::uuid[]), 'fec'::identifier_source, 'fec_candidate_id'::identifier_type, unnest($2::text[]), $3
               ON CONFLICT DO NOTHING"#,
        )
        .bind(&entity_ids)
        .bind(&fec_ids)
        .bind(evidence_id)
        .execute(pool)
        .await
        {
            warn!("FEC: candidate identifier insert failed: {e}");
        }

        // Insert candidate_metadata
        let parties: Vec<&str> = chunk.iter().map(|c| c.party.as_str()).collect();
        let offices: Vec<&str> = chunk.iter().map(|c| c.office.as_str()).collect();
        let states: Vec<&str> = chunk.iter().map(|c| c.office_st.as_str()).collect();
        let districts: Vec<&str> = chunk.iter().map(|c| c.office_district.as_str()).collect();
        let icis: Vec<&str> = chunk.iter().map(|c| c.ici.as_str()).collect();
        let statuses: Vec<&str> = chunk.iter().map(|c| c.status.as_str()).collect();
        let pccs: Vec<&str> = chunk.iter().map(|c| c.pcc.as_str()).collect();
        let election_years: Vec<Option<i32>> = chunk
            .iter()
            .map(|c| c.election_year.parse::<i32>().ok())
            .collect();

        if let Err(e) = sqlx::query(
            r#"INSERT INTO candidate_metadata
               (entity_id, candidate_fec_id, party, office, office_state, office_district,
                incumbent_challenge, election_year, candidate_status, principal_committee_fec_id)
               SELECT unnest($1::uuid[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]),
                      unnest($5::text[]), unnest($6::text[]), unnest($7::text[]), unnest($8::int[]),
                      unnest($9::text[]), unnest($10::text[])
               ON CONFLICT (entity_id) DO UPDATE SET
                 party = EXCLUDED.party, office = EXCLUDED.office,
                 office_state = EXCLUDED.office_state, office_district = EXCLUDED.office_district,
                 incumbent_challenge = EXCLUDED.incumbent_challenge,
                 election_year = EXCLUDED.election_year,
                 candidate_status = EXCLUDED.candidate_status,
                 principal_committee_fec_id = EXCLUDED.principal_committee_fec_id"#,
        )
        .bind(&entity_ids)
        .bind(&fec_ids)
        .bind(&parties)
        .bind(&offices)
        .bind(&states)
        .bind(&districts)
        .bind(&icis)
        .bind(&election_years)
        .bind(&statuses)
        .bind(&pccs)
        .execute(pool)
        .await
        {
            warn!("FEC: candidate_metadata insert failed: {e}");
        }

        for (i, c) in chunk.iter().enumerate() {
            map.insert(c.cand_id.clone(), entity_ids[i]);
        }
        summary.entities_created += chunk.len() as u64;
    }

    map
}

async fn batch_create_committees(
    pool: &PgPool,
    committees: &HashMap<String, CommitteeEntry>,
    evidence_id: Uuid,
    summary: &mut ImportSummary,
) -> HashMap<String, Uuid> {
    let mut map = HashMap::new();

    // Load existing
    let existing = load_existing_fec_ids(pool, "fec_committee_id").await;
    for (fec_id, entity_id) in &existing {
        map.insert(fec_id.clone(), *entity_id);
    }

    let new_committees: Vec<&CommitteeEntry> = committees
        .values()
        .filter(|c| !map.contains_key(&c.cmte_id))
        .collect();

    if new_committees.is_empty() {
        // Still update metadata for existing committees
        update_committee_metadata(pool, &existing, committees).await;
        return map;
    }

    for chunk in new_committees.chunks(ENTITY_BATCH) {
        let names: Vec<&str> = chunk.iter().map(|c| c.cmte_name.as_str()).collect();
        let canonicals: Vec<String> = chunk.iter().map(|c| normalize_name(&c.cmte_name)).collect();
        let can_refs: Vec<&str> = canonicals.iter().map(|s| s.as_str()).collect();
        let fec_ids: Vec<&str> = chunk.iter().map(|c| c.cmte_id.as_str()).collect();

        let rows = match sqlx::query(
            r#"INSERT INTO entity (entity_type, display_name, canonical_name)
               SELECT 'committee'::entity_type, t.name, t.canonical
               FROM unnest($1::text[], $2::text[]) AS t(name, canonical)
               RETURNING id"#,
        )
        .bind(&names)
        .bind(&can_refs)
        .fetch_all(pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("FEC: batch committee insert failed: {e}");
                summary.errors += chunk.len() as u64;
                continue;
            }
        };

        let entity_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

        if let Err(e) = sqlx::query(
            r#"INSERT INTO entity_identifier (entity_id, source, identifier_type, identifier_value, evidence_id)
               SELECT unnest($1::uuid[]), 'fec'::identifier_source, 'fec_committee_id'::identifier_type, unnest($2::text[]), $3
               ON CONFLICT DO NOTHING"#,
        )
        .bind(&entity_ids)
        .bind(&fec_ids)
        .bind(evidence_id)
        .execute(pool)
        .await
        {
            warn!("FEC: committee identifier insert failed: {e}");
        }

        let designations: Vec<&str> = chunk.iter().map(|c| c.designation.as_str()).collect();
        let cmte_types: Vec<&str> = chunk.iter().map(|c| c.cmte_tp.as_str()).collect();
        let parties: Vec<&str> = chunk.iter().map(|c| c.party.as_str()).collect();
        let treasurers: Vec<&str> = chunk.iter().map(|c| c.treasurer.as_str()).collect();
        let org_types: Vec<&str> = chunk.iter().map(|c| c.org_tp.as_str()).collect();
        let connected: Vec<&str> = chunk.iter().map(|c| c.connected_org.as_str()).collect();

        if let Err(e) = sqlx::query(
            r#"INSERT INTO committee_metadata
               (entity_id, committee_fec_id, committee_type, committee_designation, party,
                treasurer_name, organization_type, connected_organization)
               SELECT unnest($1::uuid[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]),
                      unnest($5::text[]), unnest($6::text[]), unnest($7::text[]), unnest($8::text[])
               ON CONFLICT (entity_id) DO UPDATE SET
                 committee_type = EXCLUDED.committee_type,
                 committee_designation = EXCLUDED.committee_designation,
                 party = EXCLUDED.party, treasurer_name = EXCLUDED.treasurer_name,
                 organization_type = EXCLUDED.organization_type,
                 connected_organization = EXCLUDED.connected_organization"#,
        )
        .bind(&entity_ids)
        .bind(&fec_ids)
        .bind(&cmte_types)
        .bind(&designations)
        .bind(&parties)
        .bind(&treasurers)
        .bind(&org_types)
        .bind(&connected)
        .execute(pool)
        .await
        {
            warn!("FEC: committee_metadata insert failed: {e}");
        }

        for (i, c) in chunk.iter().enumerate() {
            map.insert(c.cmte_id.clone(), entity_ids[i]);
        }
        summary.entities_created += chunk.len() as u64;
    }

    update_committee_metadata(pool, &existing, committees).await;

    map
}

async fn update_committee_metadata(
    pool: &PgPool,
    existing: &HashMap<String, Uuid>,
    committees: &HashMap<String, CommitteeEntry>,
) {
    let to_update: Vec<(&Uuid, &CommitteeEntry)> = existing
        .iter()
        .filter_map(|(fec_id, eid)| committees.get(fec_id).map(|c| (eid, c)))
        .collect();

    for chunk in to_update.chunks(ENTITY_BATCH) {
        let entity_ids: Vec<Uuid> = chunk.iter().map(|(eid, _)| **eid).collect();
        let fec_ids: Vec<&str> = chunk.iter().map(|(_, c)| c.cmte_id.as_str()).collect();
        let cmte_types: Vec<&str> = chunk.iter().map(|(_, c)| c.cmte_tp.as_str()).collect();
        let designations: Vec<&str> = chunk.iter().map(|(_, c)| c.designation.as_str()).collect();
        let parties: Vec<&str> = chunk.iter().map(|(_, c)| c.party.as_str()).collect();
        let treasurers: Vec<&str> = chunk.iter().map(|(_, c)| c.treasurer.as_str()).collect();
        let org_types: Vec<&str> = chunk.iter().map(|(_, c)| c.org_tp.as_str()).collect();
        let connected: Vec<&str> = chunk
            .iter()
            .map(|(_, c)| c.connected_org.as_str())
            .collect();

        let _ = sqlx::query(
            r#"INSERT INTO committee_metadata
               (entity_id, committee_fec_id, committee_type, committee_designation, party,
                treasurer_name, organization_type, connected_organization)
               SELECT unnest($1::uuid[]), unnest($2::text[]), unnest($3::text[]), unnest($4::text[]),
                      unnest($5::text[]), unnest($6::text[]), unnest($7::text[]), unnest($8::text[])
               ON CONFLICT (entity_id) DO UPDATE SET
                 committee_type = EXCLUDED.committee_type,
                 committee_designation = EXCLUDED.committee_designation,
                 party = EXCLUDED.party, treasurer_name = EXCLUDED.treasurer_name,
                 organization_type = EXCLUDED.organization_type,
                 connected_organization = EXCLUDED.connected_organization"#,
        )
        .bind(&entity_ids)
        .bind(&fec_ids)
        .bind(&cmte_types)
        .bind(&designations)
        .bind(&parties)
        .bind(&treasurers)
        .bind(&org_types)
        .bind(&connected)
        .execute(pool)
        .await;
    }
}

async fn batch_create_ccl_links(
    pool: &PgPool,
    links: &[CclEntry],
    cand_map: &HashMap<String, Uuid>,
    cmte_map: &HashMap<String, Uuid>,
) {
    let valid: Vec<(&CclEntry, Uuid, Uuid)> = links
        .iter()
        .filter_map(|l| {
            let cand_eid = cand_map.get(&l.cand_id)?;
            let cmte_eid = cmte_map.get(&l.cmte_id)?;
            Some((l, *cand_eid, *cmte_eid))
        })
        .collect();

    for chunk in valid.chunks(ENTITY_BATCH) {
        let cand_ids: Vec<&str> = chunk.iter().map(|(l, _, _)| l.cand_id.as_str()).collect();
        let cmte_ids: Vec<&str> = chunk.iter().map(|(l, _, _)| l.cmte_id.as_str()).collect();
        let cand_eids: Vec<Uuid> = chunk.iter().map(|(_, eid, _)| *eid).collect();
        let cmte_eids: Vec<Uuid> = chunk.iter().map(|(_, _, eid)| *eid).collect();
        let designations: Vec<&str> = chunk
            .iter()
            .map(|(l, _, _)| l.cmte_designation.as_str())
            .collect();

        if let Err(e) = sqlx::query(
            r#"INSERT INTO candidate_committee_link
               (candidate_entity_id, committee_entity_id, candidate_fec_id, committee_fec_id, committee_designation)
               SELECT unnest($1::uuid[]), unnest($2::uuid[]), unnest($3::text[]), unnest($4::text[]), unnest($5::text[])
               ON CONFLICT (candidate_fec_id, committee_fec_id) DO UPDATE SET
                 committee_designation = EXCLUDED.committee_designation"#,
        )
        .bind(&cand_eids)
        .bind(&cmte_eids)
        .bind(&cand_ids)
        .bind(&cmte_ids)
        .bind(&designations)
        .execute(pool)
        .await
        {
            warn!("FEC: ccl link insert failed: {e}");
        }

        // Create candidate_supported_by_committee edges
        if let Err(e) = sqlx::query(
            r#"INSERT INTO relationship_edge
               (from_entity_id, to_entity_id, edge_type, amount, currency, confidence, source, started_on)
               SELECT unnest($1::uuid[]), unnest($2::uuid[]),
                      'candidate_supported_by_committee'::edge_type,
                      NULL, 'USD', 1.0, 'fec', NULL
               ON CONFLICT DO NOTHING"#,
        )
        .bind(&cmte_eids) // from = committee
        .bind(&cand_eids) // to = candidate
        .execute(pool)
        .await
        {
            warn!("FEC: ccl edge insert failed: {e}");
        }
    }
}

async fn load_existing_fec_ids(pool: &PgPool, id_type: &str) -> HashMap<String, Uuid> {
    let rows = sqlx::query(
        "SELECT identifier_value, entity_id FROM entity_identifier WHERE identifier_type = $1::identifier_type",
    )
    .bind(id_type)
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

fn merge_summary(into: &mut ImportSummary, from: &ImportSummary) {
    into.records_imported += from.records_imported;
    into.records_skipped += from.records_skipped;
    into.entities_created += from.entities_created;
    into.errors += from.errors;
}
