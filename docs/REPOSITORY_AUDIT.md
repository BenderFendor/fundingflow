# FundingFlow repository audit

## Intended product

FundingFlow is a source-backed public-record graph. Its core product loop is:

1. Ingest official spending, lobbying, campaign-finance, corporate-disclosure, economic, and rulemaking records.
2. Resolve source identifiers into canonical people, organizations, committees, agencies, and geographic entities.
3. Preserve evidence and provenance for each normalized record and relationship.
4. Serve search, profile, metric, and relationship views without making unsupported claims about causation or wrongdoing.

## Existing implementation retained

The repository already contains the major application skeleton and substantial working code:

- A Rust workspace with domain, database, ingestion, API, and CLI crates.
- SQLx migrations and PostgreSQL query modules.
- Importers for USAspending, LDA, FEC, SEC, Federal Register, BLS, Census, and related economic data.
- Next.js pages for national conditions, states, counties, entities, data lineage, methodology, and cost baskets.
- Unit and ignored database integration tests.
- A local self-test covering Rust formatting, compilation, clippy, tests, frontend lint, typecheck, and build.

## Defects repaired in this change

### Browser API routing

The interactive home search imported a shared API client whose fallback origin was `http://localhost:3001`. In a deployed browser, that address refers to the visitor's computer. The search now uses a relative same-origin endpoint, and a Next.js route handler proxies it to the internal Rust API using `API_URL`.

### Frozen frontend installation

`package.json` requested `@eslint/js` `^9.39.4`, while `pnpm-lock.yaml` recorded `^10.0.1`. A frozen pnpm install therefore rejected the repository state. The manifest now matches the committed lockfile.

### API boundary validation

Search, pagination, history, campaign-cycle, threshold, and committee parameters previously accepted negative, empty, invalid, or effectively unbounded values. Those values could produce PostgreSQL errors or wasteful queries. The API now rejects malformed requests before they reach the database and includes regression tests for the validation rules.

### Operational readiness

The API now distinguishes process liveness (`/health`) from database readiness (`/ready`), logs query failures, restricts CORS to an explicit origin by default, and shuts down gracefully on SIGINT or SIGTERM.

### Delivery path

The repository now includes:

- GitHub Actions gates for Rust, database integration tests, Next.js, and container builds.
- Production Dockerfiles for the Rust and Next.js services.
- A Compose stack with PostgreSQL health checks, automatic migrations, API readiness ordering, persistent storage, and a web-facing same-origin proxy.
- Deployment, import, backup, restore, and environment documentation.

## Remaining product work

The application is runnable and testable after this change, but completion of the full public-record product still depends on data coverage rather than missing scaffolding. The largest remaining workstreams are:

- Run and measure complete source backfills rather than small importer limits.
- Add source-level coverage tables and expose failed, stale, unresolved, and unpublished records in the UI.
- Replace heuristic name-only matches with explicit resolution decisions, confidence evidence, and review queues.
- Add curated multi-hop graph and timeline views instead of relying only on entity relationship lists.
- Add reproducible data snapshots and fixture-based importer regression tests for every source schema.
- Add authentication only if private administrative import controls are exposed; the public product itself should remain account-free.

These are intentionally documented as coverage and product milestones rather than hidden behind placeholder data or claims of completeness.
