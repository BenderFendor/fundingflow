# FundingFlow Agent Rules

## Frontend Design Rules

- Before frontend design work, read `docs/agent/DESIGN.md`.
- Keep `docs/agent/DESIGN.md` current when website design principles change.

## Quick Verification

```bash
# Full self-test (Rust + Frontend)
./verify.sh

# API health check
curl -s http://localhost:3001/health | jq

# Check database state
psql -h localhost -U fundingflow -d fundingflow -c "
SELECT entity_type, COUNT(*) FROM entity GROUP BY entity_type ORDER BY count DESC;
SELECT 'awards' as tbl, COUNT(*) FROM award
UNION ALL SELECT 'edges', COUNT(*) FROM relationship_edge
UNION ALL SELECT 'lobbying', COUNT(*) FROM lobbying_filing
UNION ALL SELECT 'metrics', COUNT(*) FROM metric_observation;
"
```

## API Endpoint Tests

```bash
# Search entities
curl -s "http://localhost:3001/api/v1/entities/search?q=Boeing&limit=5" | jq '.total, .entities[].display_name'

# Search people
curl -s "http://localhost:3001/api/v1/entities/search?q=Elon&limit=5" | jq

# Fuzzy search
curl -s "http://localhost:3001/api/v1/entities/search/fuzzy?q=musk&threshold=0.1&limit=5" | jq

# Geo search
curl -s "http://localhost:3001/api/v1/geos/search?q=California&limit=5" | jq

# Unified search (entities + geos)
curl -s "http://localhost:3001/api/v1/search?q=California&limit=5" | jq

# National pulse
curl -s "http://localhost:3001/api/v1/pulse/national" | jq '.latest_metrics | length'

# State profile
curl -s "http://localhost:3001/api/v1/geos/CA/profile" | jq '{geo: .geo.name, metrics: (.latest_metrics | length), derived: (.derived_metrics | length)}'

# Entity detail
curl -s "http://localhost:3001/api/v1/entities/search?q=Lockheed&limit=1" | jq -r '.entities[0].id' | xargs -I{} curl -s "http://localhost:3001/api/v1/entities/{}" | jq '{name: .entity.display_name, ids: (.identifiers | length)}'

# Entity awards
curl -s "http://localhost:3001/api/v1/entities/search?q=Boeing&limit=1" | jq -r '.entities[0].id' | xargs -I{} curl -s "http://localhost:3001/api/v1/entities/{}/awards" | jq 'length'

# Entity lobbying
curl -s "http://localhost:3001/api/v1/entities/search?q=Boeing&limit=1" | jq -r '.entities[0].id' | xargs -I{} curl -s "http://localhost:3001/api/v1/entities/{}/lobbying" | jq 'length'

# Entity edges
curl -s "http://localhost:3001/api/v1/entities/search?q=Boeing&limit=1" | jq -r '.entities[0].id' | xargs -I{} curl -s "http://localhost:3001/api/v1/entities/{}/edges" | jq 'length'

# Entity economic context
curl -s "http://localhost:3001/api/v1/entities/search?q=LOCKHEED&limit=1" | jq -r '.entities[0].id' | xargs -I{} curl -s "http://localhost:3001/api/v1/entities/{}/economic-context" | jq 'length'
```

## Importing Data

```bash
# Rebuild
cargo build --release --bin api --bin cli

# Run all importers (USASpending with pagination imports ~5000 awards)
./runlocal.sh import-usaspending
./runlocal.sh import-lda
./runlocal.sh import-sec
./runlocal.sh import-rulemaking

# Economic data imports
./target/release/cli import --source economic-fixture
./target/release/cli import --source bls-laus
./target/release/cli import --source bls-cpi-prices
./target/release/cli import --source census-acs
./target/release/cli derive --metric state-mvp
```

## Data Requirements

- USASpending importer paginates (100/page, 50 max pages). Uses upsert for source records and awards.
- Place of performance geo IDs extracted from state names (converted to 2-letter codes) or place_of_performance text.
- DC is included in geo table alongside 50 states.
- Economic fixtures populate metrics for all 51 geos (50 states + DC + US).
- Frontend entity detail page has 4 tabs: Awards, Lobbying, Relationships, Economic Context.
- Unified search (/api/v1/search) returns both entities and geos.
- Entity economic context endpoint returns full state profiles for all geos linked to an entity's awards.
