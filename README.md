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

- Current stable Rust toolchain with `rustfmt` and `clippy`
- PostgreSQL 16+
- Node.js 22+
- pnpm 10.13.1 through Corepack or a compatible pnpm 10 installation

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
corepack enable
pnpm install --frozen-lockfile
pnpm dev
```

The development frontend is available at `http://localhost:3000`. Browser-side API requests use the same-origin Next.js proxy at `/api/v1/*`, which forwards to `API_URL` or `NEXT_PUBLIC_API_URL` on the server.

## Verification

Run the complete local gate:

```bash
./scripts/self-test
```

The gate checks Rust formatting, compilation, clippy, unit tests, frontend lint, TypeScript, and the production Next.js build. Database integration tests use the disposable test stack:

```bash
./scripts/test-integration
```

Pull requests run the same core checks in GitHub Actions and also build both production container images.

## Product surfaces

- Unified entity and geography search
- Entity profiles with identifiers, aliases, awards, lobbying, relationships, economic context, and campaign finance
- State profiles with labor, wages, housing, food, energy, income, and public-money context
- National cost basket and household-pressure indicators
- Data-lineage and methodology documentation
- Source-backed evidence and neutral relationship labels

## API examples

```bash
curl "http://localhost:3001/api/v1/search?q=California&limit=5"
curl "http://localhost:3001/api/v1/entities/search?q=Lockheed&limit=5"
curl "http://localhost:3001/api/v1/pulse/national"
curl "http://localhost:3001/api/v1/geos/PA/profile"
```

See `AGENTS.md` for a broader endpoint checklist and importer commands.

## Interpretation

FundingFlow shows relationships documented in public records. These links do not prove causation, intent, policy influence, favoritism, or wrongdoing. Obligated federal amounts are not recognized company revenue, and national or regional economic observations are not silently presented as local data.

## License

MIT
