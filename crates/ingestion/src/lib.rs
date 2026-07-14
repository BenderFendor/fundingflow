use anyhow::Result;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub mod economic;
pub mod fec;
pub mod fec_txn;
pub mod lda;
pub mod openfec_api;
pub mod rulemaking;
pub mod sec;
pub mod usaspending;

#[async_trait]
pub trait Importer: Send + Sync {
    fn source_name(&self) -> &'static str;

    async fn import(&self, pool: &PgPool) -> Result<ImportSummary>;
}

#[derive(Debug, Clone)]
pub struct ImportSummary {
    pub source: String,
    pub records_imported: u64,
    pub records_skipped: u64,
    pub entities_created: u64,
    pub errors: u64,
}

pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn normalize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name_lowercase() {
        assert_eq!(normalize_name("TESLA INC"), "tesla inc");
    }

    #[test]
    fn test_normalize_name_whitespace() {
        assert_eq!(normalize_name("  Tesla   Inc  "), "tesla inc");
    }

    #[test]
    fn test_normalize_name_single_word() {
        assert_eq!(normalize_name("  SpaceX  "), "spacex");
    }

    #[test]
    fn test_normalize_name_with_punctuation() {
        assert_eq!(
            normalize_name("Lockheed Martin Corp."),
            "lockheed martin corp."
        );
    }

    #[test]
    fn test_content_hash_deterministic() {
        let h1 = content_hash("hello");
        let h2 = content_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_different_inputs() {
        let h1 = content_hash("hello");
        let h2 = content_hash("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_length() {
        let hash = content_hash("some content");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_import_summary_defaults() {
        let summary = ImportSummary {
            source: "test".to_string(),
            records_imported: 0,
            records_skipped: 0,
            entities_created: 0,
            errors: 0,
        };
        assert_eq!(summary.records_imported, 0);
    }
}
