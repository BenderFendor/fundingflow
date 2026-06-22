# FundingFlow Agent Rules

- Keep the Rust backend as the source of truth for ingestion, evidence, metrics, and API responses.
- Store source records and evidence before deriving entities, edges, observations, or metrics.
- Show missing data as missing. Do not convert unavailable source values to zero.
- Preserve `release_date` and `vintage_date` for economic observations.
- Use neutral labels for public-record relationships and avoid causal claims.
- Run `./scripts/self-test` before handoff unless a blocker is documented.
