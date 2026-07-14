# FundingFlow deployment

FundingFlow ships as three long-running services plus a one-shot migration job:

- PostgreSQL 16 stores normalized records, evidence, metrics, and graph edges.
- The Rust API serves public search and entity endpoints on port 3001.
- The Next.js application serves the interface on port 3000 and proxies browser API requests through `/api/v1/*`.
- The migration job runs the Rust CLI before the API starts.

## Start the complete stack

Copy the environment template and replace the database password and SEC contact identity before importing public records.

```bash
cp .env.example .env
# Edit POSTGRES_PASSWORD and SEC_USER_AGENT.
docker compose up --build -d
```

Open `http://localhost:3000`. The API remains bound to `127.0.0.1:3001` by default, while the web application exposes a same-origin API proxy for browser searches.

Check service state:

```bash
docker compose ps
curl --fail http://localhost:3001/health
curl --fail http://localhost:3001/ready
```

`/health` proves the API process is alive. `/ready` also verifies that PostgreSQL accepts queries.

## Import real public records

The production stack does not seed demonstration entities. Run the source importers against the persistent database:

```bash
docker compose run --rm api cli import --source usaspending
docker compose run --rm api cli import --source lda
docker compose run --rm api cli import --source sec
docker compose run --rm api cli import --source rulemaking
```

Economic imports and derived metrics:

```bash
docker compose run --rm api cli import --source bls-laus
docker compose run --rm api cli import --source bls-cpi-prices
docker compose run --rm api cli import --source census-acs
docker compose run --rm api cli derive --metric state-mvp
```

FEC imports can require `FEC_API_KEY`, depending on the selected importer. Regulations.gov ingestion requires `REGULATIONS_GOV_API_KEY`. SEC requests must use a real identifying `SEC_USER_AGENT` with a monitored contact email.

## Update the application

```bash
git pull
docker compose build
docker compose up -d
```

The `migrate` service runs before each new API container and applies pending SQLx migrations. PostgreSQL data remains in the `fundingflow-postgres` named volume.

## Back up and restore PostgreSQL

Create a compressed logical backup:

```bash
docker compose exec -T db pg_dump -U fundingflow -d fundingflow -Fc > fundingflow.dump
```

Restore into an empty database:

```bash
docker compose exec -T db pg_restore \
  -U fundingflow \
  -d fundingflow \
  --clean \
  --if-exists < fundingflow.dump
```

Stop the application without deleting data:

```bash
docker compose down
```

Delete the database volume only when a full reset is intentional:

```bash
docker compose down --volumes
```

## Environment reference

| Variable | Service | Purpose |
| --- | --- | --- |
| `DATABASE_URL` | API and CLI | PostgreSQL connection string. |
| `POSTGRES_PASSWORD` | Compose | Password used by PostgreSQL and the application services. |
| `API_URL` | Next.js server | Internal API origin used by the same-origin browser proxy. |
| `NEXT_PUBLIC_API_URL` | Next.js build | API origin used by server-rendered data functions already in the application. |
| `CORS_ALLOWED_ORIGIN` | API | The one direct browser origin allowed to call the Rust API. Use `*` only for intentionally public cross-origin access. |
| `SEC_USER_AGENT` | SEC importer | Identifying application and contact required by SEC fair-access policy. |
| `FEC_API_KEY` | FEC importer | Optional OpenFEC API credential. |
| `REGULATIONS_GOV_API_KEY` | Regulations.gov importer | API credential for docket and comment data. |
| `WEB_PORT` | Compose | Host port for the Next.js application. |
| `API_PORT` | Compose | Loopback host port for direct API diagnostics. |

## Production hardening

Place TLS termination in front of port 3000, use a generated database password, restrict host firewall access to PostgreSQL and the direct API port, and send container logs to persistent storage. Keep the API behind the web proxy unless direct public API access is intentional.
