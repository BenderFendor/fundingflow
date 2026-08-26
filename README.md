# FundingFlow

FundingFlow turns scattered federal public records into one searchable graph. It imports federal spending from USAspending, lobbying filings from LDA, campaign finance from the FEC, corporate disclosures from SEC EDGAR, rulemaking documents from the Federal Register, and state-level economic indicators from BLS, BEA, Census, HUD, EIA, and FHFA. A Rust API serves the joined graph as JSON and a Next.js app browses it. Every displayed relationship cites the source record it came from, and the app treats those links as documented facts, not conclusions about intent.

## Status

Maintenance. Last commit 2026-07-14; developed in bursts between other projects.

## Features

- Import public records from USAspending, LDA, FEC bulk files, OpenFEC Schedule E, SEC EDGAR, and the Federal Register.
- Import 11 economic datasets (BLS, BEA, Census ACS, HUD FMR, EIA, FHFA HPI, DOL minimum wage) for all 50 states, DC, and the nation.
- Resolve entities across sources using deterministic identifiers (UEI, CIK, EIN, FEC IDs) plus pg_trgm fuzzy matching over names and aliases.
- Store typed relationship edges, each backed by evidence rows pointing at the source record.
- Search entities and geographies with exact, fuzzy, and unified (entities plus geos) queries.
- Serve a JSON API covering entity awards, lobbying, edges, economic context, geo profiles, contribution flows, and a national pulse.
- Browse the graph in a Next.js UI: unified search, entity detail tabs, state and county pages, a cost basket view, and a methodology page.
- Derive state-level metric sets from imported observations with `cli derive --metric state-mvp`.
- Run the whole stack locally without Docker via `./runlocal.sh`.

## Screenshots

![Home page search returning person entities with their identifiers](clipboard.png)

## Stack

| Layer | Choice |
|-------|--------|
| Backend | Rust (2024 edition), axum 0.8, tokio, sqlx 0.8, clap 4 |
| Database | PostgreSQL 16 with pg_trgm and uuid-ossp extensions |
| Frontend | Next.js App Router, React, TypeScript strict mode, Tailwind CSS 4 |
| Tooling | Cargo workspace, pnpm 10 workspace (apps/*), ESLint 9, ast-grep rules |

## Requirements

- Rust 1.85+ (the workspace builds with edition 2024)
- PostgreSQL 16+ with pg_trgm and uuid-ossp (both ship in postgresql-contrib; the test compose file pins postgres:16-alpine)
- pnpm 10.25.0, pinned via packageManager; `corepack enable` provides it
- Node.js for the frontend; the repo pins no Node version
- Docker, only for the integration test database
- API keys only for the four economic sources that require them (see Configuration)

## Install

```bash
git clone <your-fork-url> fundingflow
cd fundingflow
corepack enable
pnpm install
```

Create the database and role:

```bash
psql -U postgres -c "CREATE ROLE fundingflow WITH LOGIN PASSWORD 'fundingflow';"
psql -U postgres -c "CREATE DATABASE fundingflow OWNER fundingflow;"
```

Copy `.env.example` to `.env` and adjust. The Rust binaries load it through dotenvy; every default matches the commands below.

## Configuration

Every environment variable the code reads:

| Variable | Read by | Default | Meaning |
|----------|---------|---------|---------|
| `DATABASE_URL` | api, cli, db | `postgres://fundingflow:fundingflow@localhost:5432/fundingflow` | Postgres connection string. The CLI also accepts `--database-url`. |
| `PORT` | api | `3001` | API listen port |
| `RUST_LOG` | api, cli | `info` | Tracing filter |
| `NEXT_PUBLIC_API_URL` | apps/web | `http://localhost:3001` | API base URL, read at build time |
| `FEC_API_KEY` | ingestion (openfec-schedule-e) | `DEMO_KEY` | OpenFEC API key; DEMO_KEY has tight rate limits |
| `CENSUS_API_KEY` | ingestion (census-acs) | unset | Without it the import skips live observations |
| `HUD_FMR_API_KEY` | ingestion (hud-fmr) | unset | Same skip behavior when unset |
| `EIA_API_KEY` | ingestion (eia-gas) | unset | Same skip behavior when unset |
| `BEA_API_KEY` | ingestion (bea-regional) | unset | Same skip behavior when unset |
| `TEST_DATABASE_URL` | db integration tests | same as `DATABASE_URL` default | Database used by `scripts/test-integration` |

`runlocal.sh` reads its own overrides: `BACKEND_PORT` (3001), `FRONTEND_PORT` (3000), `POSTGRES_HOST`, `POSTGRES_PORT`, `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`, `DATABASE_URL`, `NEXT_PUBLIC_API_URL`, `LOG_DIR`, `AUTO_INSTALL`.

## Run

Apply the schema and load data:

```bash
cargo run --bin cli -- migrate
cargo run --bin cli -- seed                        # optional sample data
cargo run --bin cli -- import --source usaspending
cargo run --bin cli -- import --source lda
cargo run --bin cli -- import --source sec
cargo run --bin cli -- import --source rulemaking
```

Import sources: `usaspending`, `lda`, `fec`, `openfec-schedule-e`, `sec`, `rulemaking`, `economic-fixture`, plus the economic set `bls-laus`, `bls-ces`, `bls-cpi-prices`, `bls-cps`, `dol-min-wage`, `bea-regional`, `census-acs`, `fhfa-hpi`, `hud-fmr`, `eia-gas`, `usaspending-state`. Derived metrics:

```bash
cargo run --bin cli -- derive --metric state-mvp
```

Start the two servers:

```bash
cargo run --bin api          # API on http://localhost:3001
cd apps/web && pnpm dev      # web on http://localhost:3000
```

`./runlocal.sh` does all of it in one command (Postgres check, migrations, backend, frontend). Subcommands: `setup`, `services`, `backend`, `frontend`, `all` (default), `migrate`, `seed`, `import-<source>`, `killall`.

## Examples

```bash
# Unified search across entities and geographies
curl -s "http://localhost:3001/api/v1/search?q=California&limit=5" | jq

# Fuzzy entity search tolerant of typos
curl -s "http://localhost:3001/api/v1/entities/search/fuzzy?q=musk&threshold=0.1&limit=5" | jq

# National pulse: latest economic metrics and derived indicators
curl -s http://localhost:3001/api/v1/pulse/national | jq

# State profile with latest and derived metrics
curl -s http://localhost:3001/api/v1/geos/CA/profile | jq
```

## Architecture

Cargo workspace with five crates and a Next.js app:

```
crates/
├── domain/      shared types: entities, identifiers, edges, awards, filings, metrics
├── db/          sqlx pool, query layer, and the three SQL migrations in crates/db/migrations
├── ingestion/   one importer module per source plus the economic importer family
├── api/         axum HTTP server, all routes under /api/v1
└── cli/         migrate, seed, import, derive subcommands
apps/web/        Next.js App Router UI (search, entity detail, state and county pages)
```

Data flow: importers fetch from the public APIs and upsert into `source_record` (unique per source, type, and record id), so re-imports are idempotent. They create or link `entity` rows using deterministic identifiers and pg_trgm name matching, with `identity_match` tracking candidate and accepted merges. Typed rows land in `award`, `lobbying_filing`, `campaign_finance_transaction`, SEC fact, and rulemaking tables, and `relationship_edge` rows connect entities with evidence pointing at the source record. Economic importers fill `geo`, `data_source`, `metric`, and `metric_observation`; `derive --metric state-mvp` writes `derived_metric_observation`. The API is read-only over these tables and the frontend calls it through `NEXT_PUBLIC_API_URL`.

## Testing

Unit tests and static checks, no database needed:

```bash
cargo test --workspace --all-features
./scripts/self-test    # fmt, check, clippy -D warnings, cargo test, web lint/typecheck/build
```

Database integration tests (20 tests in `crates/db/tests`, marked `#[ignore = "requires database"]`):

```bash
./scripts/test-integration   # disposable Postgres 16 via docker compose on :5433, single-threaded run
```

There is no CI. The repo has no GitHub Actions workflows, so nothing runs these commands automatically on push; `./scripts/self-test` is the local gate and the natural seed for a future workflow.

## Known limits

- Imports and derives are manual CLI runs. Nothing schedules or refreshes data; a code comment in the CLI already flags that this should become one process instead of CLI plus API.
- FEC imports target the 2024 cycle, and OpenFEC falls back to the rate-limited `DEMO_KEY` without `FEC_API_KEY`.
- The USAspending importer paginates 100 records per page with a 50 page cap, so an import is a broad sample, not the complete award history.
- CORS is fully permissive. Fine for local use; not hardened for public hosting.
- Entity matching beyond deterministic identifiers and pg_trgm (the `splink` method in the schema) is defined but not automated.

## Credits

Data comes from public sources: USAspending.gov, LDA.gov, FEC.gov bulk downloads and the OpenFEC API, SEC EDGAR, the Federal Register, BLS (LAUS, CES, CPI, CPS), BEA regional accounts, Census ACS, HUD Fair Market Rents, EIA weekly petroleum prices, FHFA House Price Index, and DOL minimum wage history.

## License

None chosen yet (deciding between MIT and Apache-2.0); the workspace Cargo.toml declares `license = "MIT"` but no LICENSE file exists.
