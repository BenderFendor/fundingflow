# FundingFlow

Public-record graph for federal spending, lobbying, campaign finance, corporate disclosures, rulemaking records, and economic conditions.

FundingFlow maps public records into a searchable graph of federal money, lobbying activity, political finance, company disclosures, and rulemaking participation. Every edge and aggregate is source-backed with evidence citations.

## Stack

- **Backend**: Rust with axum, sqlx, tokio, and serde
- **Database**: PostgreSQL 16 with `pg_trgm` for fuzzy entity matching
- **Frontend**: Next.js with TypeScript strict mode
- **CLI and ingestion**: Rust with source-specific importers
- **Deployment**: Docker Compose with automatic migrations and health checks

## Fastest complete setup

The Compose stack starts PostgreSQL, applies migrations, starts the Rust API, and starts the Next.js application.

```bash
cp .env.example .env
# Replace POSTGRES_PASSWORD and SEC_USER_AGENT in .env.
docker compose up --build -d
```

Open `http://localhost:3000`.

Verify both API liveness and database readiness:

```bash
curl --fail http://localhost:3001/health
curl --fail http://localhost:3001/ready
```

The stack does not populate demonstration records. Import real public data with the CLI:

```bash
docker compose run --rm api cli import --source usaspending
docker compose run --rm api cli import --source lda
docker compose run --rm api cli import --source sec
docker compose run --rm api cli import --source rulemaking
```

See [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) for economic imports, credentials, backups, restores, and production operation. See [`docs/REPOSITORY_AUDIT.md`](docs/REPOSITORY_AUDIT.md) for the product inventory and remaining coverage work.

## Local development

### Prerequisites

- Rust 1.85+
- PostgreSQL 16+
- Node.js 22+
- pnpm 10+

### Database and API

```bash
psql -U postgres -c "CREATE ROLE fundingflow WITH LOGIN PASSWORD 'fundingflow';"
psql -U postgres -c "CREATE DATABASE fundingflow OWNER fundingflow;"

DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow \
  cargo run --bin cli -- migrate

DATABASE_URL=postgres://fundingflow:fundingflow@localhost:5432/fundingflow \
  cargo run --bin api
```

### Frontend

```bash
cd apps/web
pnpm install --frozen-lockfile
API_URL=http://localhost:3001 \
NEXT_PUBLIC_API_URL=http://localhost:3001 \
pnpm dev
```

Browser search requests use the same-origin Next.js `/api/v1/*` proxy. Server-rendered pages use the configured backend origin directly.

## Verification

Run the complete local repository gate:

```bash
./scripts/self-test
```

Run database integration tests with a disposable PostgreSQL container:

```bash
./scripts/test-integration
```

GitHub Actions runs Rust formatting, compilation, clippy, unit tests, database integration tests, frontend lint, TypeScript checks, the production Next.js build, and both container builds.

## Data sources

| Source | Description |
| --- | --- |
| USAspending.gov | Federal awards, recipients, agencies, award amounts, and performance locations |
| LDA.gov | Lobbying registrations, quarterly filings, clients, registrants, lobbyists, and issues |
| FEC.gov / OpenFEC | Committees, candidates, contributions, transfers, and campaign-finance records |
| SEC EDGAR | Company filings, XBRL facts, CIK identifiers, and disclosed subsidiaries |
| Federal Register | Rulemaking documents, agencies, publication dates, and docket references |
| BLS and Census | Labor, prices, wages, housing, population, and state/county economic conditions |

## API endpoints

| Method | Path | Description |
| --- | --- | --- |
| GET | `/health` | Process liveness check |
| GET | `/ready` | Database readiness check |
| GET | `/api/v1/search?q=...&limit=...&offset=...` | Unified entity and geography search |
| GET | `/api/v1/entities/search?q=...&limit=...&offset=...` | Entity search |
| GET | `/api/v1/entities/search/fuzzy?q=...&threshold=...` | Fuzzy entity search |
| GET | `/api/v1/entities/{id}` | Entity with identifiers and aliases |
| GET | `/api/v1/entities/{entity_id}/awards` | Awards for an entity |
| GET | `/api/v1/entities/{entity_id}/lobbying` | Lobbying filings for an entity |
| GET | `/api/v1/entities/{entity_id}/edges` | Relationship edges for an entity |
| GET | `/api/v1/entities/{entity_id}/contributions` | Campaign-finance summary and transactions |
| GET | `/api/v1/geos/{geo_id}/profile` | State or county profile |
| GET | `/api/v1/geos/{geo_id}/metrics` | Latest economic observations |
| GET | `/api/v1/geos/{geo_id}/public-money` | Public spending summary |
| GET | `/api/v1/pulse/national` | National economic pulse |
| GET | `/api/v1/contributions/search` | Contributor search |
| GET | `/api/v1/candidates/search` | Candidate search |
| GET | `/api/v1/contributions/flow` | Source-to-candidate money flow |
| GET | `/api/v1/contributions/top-donors` | Top donors for a committee |

## Relationship edge types

- `recipient_received_federal_obligation`
- `agency_obligated_award_to_recipient`
- `registrant_lobbied_for_client`
- `lobbyist_listed_on_filing`
- `client_reported_lobbying_issue`
- `person_contributed_to_committee`
- `committee_received_contribution`
- `employer_reported_on_contribution`
- `company_filed_sec_report`
- `company_reported_revenue_fact`
- `company_disclosed_subsidiary`
- `agency_published_rulemaking_document`
- `entity_submitted_rulemaking_comment`

## Disclaimer

FundingFlow shows relationships documented in public records. These links do not prove causation, intent, policy influence, favoritism, or wrongdoing.

## License

MIT
