# FundingFlow

Public-record graph for federal spending, lobbying, campaign finance, corporate disclosures, and rulemaking records.

FundingFlow maps public records into a searchable graph of federal money, lobbying activity, political finance, company disclosures, and rulemaking participation. Every edge and aggregate is source-backed with evidence citations.

## Stack

- **Backend**: Rust (axum, sqlx, tokio, serde)
- **Database**: PostgreSQL with pg_trgm for fuzzy entity matching
- **Frontend**: Next.js with TypeScript strict mode
- **CLI**: Rust (clap)

## Quick Start

### Prerequisites

- Rust 1.85+
- PostgreSQL 16+
- Node.js 22+ and pnpm

### Setup

```bash
# Create database
psql -U postgres -c "CREATE ROLE fundingflow WITH LOGIN PASSWORD 'fundingflow';"
psql -U postgres -c "CREATE DATABASE fundingflow OWNER fundingflow;"

# Run migrations
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- migrate

# Seed sample data
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- seed

# Import public records
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- import --source usaspending
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- import --source lda
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- import --source sec
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin cli -- import --source rulemaking

# Start API
DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow cargo run --bin api

# Start frontend (in another terminal)
cd apps/web && pnpm install && pnpm dev
```

### Verification

```bash
./scripts/self-test
```

## Data Sources

| Source | Description |
|--------|-------------|
| USAspending.gov | Federal awards, recipients, agencies, award amounts |
| LDA.gov | Lobbying registrations, quarterly filings, clients, issues |
| FEC.gov | Campaign committee contributions (bulk file import) |
| SEC EDGAR | Company filings, XBRL facts, CIK identifiers |
| Federal Register | Rulemaking documents, agencies, publication dates |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/v1/entities/search?q=...&limit=...&offset=...` | Search entities |
| GET | `/api/v1/entities/{id}` | Entity with identifiers and aliases |
| GET | `/api/v1/entities/{entity_id}/awards` | Awards for an entity |
| GET | `/api/v1/entities/{entity_id}/lobbying` | Lobbying filings for an entity |
| GET | `/api/v1/entities/{entity_id}/edges` | Relationship edges for an entity |

## Relationship Edge Types

- `recipient_received_federal_obligation`
- `agency_obligated_award_to_recipient`
- `registrant_lobbied_for_client`
- `person_contributed_to_committee`
- `employer_reported_on_contribution`
- `company_filed_sec_report`
- `company_reported_revenue_fact`
- `agency_published_rulemaking_document`

## Disclaimer

FundingFlow shows relationships documented in public records. These links do not prove causation, intent, policy influence, or wrongdoing.

## License

MIT
