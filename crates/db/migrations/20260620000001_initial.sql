CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE entity_type AS ENUM ('organization', 'person', 'committee', 'agency', 'unknown');
CREATE TYPE identifier_source AS ENUM ('usaspending', 'sam', 'lda', 'fec', 'sec', 'federal_register', 'regulations_gov');
CREATE TYPE identifier_type AS ENUM ('uei', 'cik', 'ein', 'fec_committee_id', 'lda_registrant_id', 'lda_client_id', 'lda_lobbyist_id', 'agency_slug', 'docket_id', 'ticker', 'award_id');
CREATE TYPE match_status AS ENUM ('candidate', 'accepted', 'rejected', 'needs_review');
CREATE TYPE match_method AS ENUM ('deterministic_identifier', 'pg_trgm', 'splink', 'dedupe', 'manual');
CREATE TYPE edge_type AS ENUM (
    'recipient_received_federal_obligation',
    'agency_obligated_award_to_recipient',
    'registrant_lobbied_for_client',
    'lobbyist_listed_on_filing',
    'client_reported_lobbying_issue',
    'committee_received_contribution',
    'person_contributed_to_committee',
    'employer_reported_on_contribution',
    'lobbyist_disclosed_contribution_to_committee',
    'company_filed_sec_report',
    'company_reported_revenue_fact',
    'company_disclosed_subsidiary',
    'agency_published_rulemaking_document',
    'entity_submitted_rulemaking_comment'
);

CREATE TABLE entity (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_type entity_type NOT NULL DEFAULT 'unknown',
    display_name TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_entity_display_name ON entity (display_name);
CREATE INDEX idx_entity_canonical_name ON entity (canonical_name);
CREATE INDEX idx_entity_display_name_trgm ON entity USING gin (display_name gin_trgm_ops);
CREATE INDEX idx_entity_canonical_name_trgm ON entity USING gin (canonical_name gin_trgm_ops);

CREATE TABLE entity_identifier (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    source identifier_source NOT NULL,
    identifier_type identifier_type NOT NULL,
    identifier_value TEXT NOT NULL,
    valid_from DATE,
    valid_to DATE,
    evidence_id UUID
);

CREATE INDEX idx_entity_identifier_entity_id ON entity_identifier (entity_id);
CREATE INDEX idx_entity_identifier_value ON entity_identifier (identifier_value);
CREATE UNIQUE INDEX idx_entity_identifier_unique ON entity_identifier (source, identifier_type, identifier_value);

CREATE TABLE entity_alias (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    alias TEXT NOT NULL,
    normalized_alias TEXT NOT NULL,
    source identifier_source NOT NULL,
    evidence_id UUID
);

CREATE INDEX idx_entity_alias_entity_id ON entity_alias (entity_id);
CREATE INDEX idx_entity_alias_normalized ON entity_alias (normalized_alias);
CREATE INDEX idx_entity_alias_normalized_trgm ON entity_alias USING gin (normalized_alias gin_trgm_ops);

CREATE TABLE identity_match (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    left_entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    right_entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    status match_status NOT NULL DEFAULT 'candidate',
    confidence DOUBLE PRECISION,
    method match_method NOT NULL,
    features_json JSONB,
    reviewed_by TEXT,
    reviewed_at TIMESTAMPTZ
);

CREATE INDEX idx_identity_match_left ON identity_match (left_entity_id);
CREATE INDEX idx_identity_match_right ON identity_match (right_entity_id);
CREATE INDEX idx_identity_match_status ON identity_match (status);

CREATE TABLE source_record (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source TEXT NOT NULL,
    source_record_type TEXT NOT NULL,
    source_record_id TEXT NOT NULL,
    source_url TEXT,
    retrieved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    content_hash TEXT NOT NULL,
    raw_storage_path TEXT
);

CREATE INDEX idx_source_record_source ON source_record (source, source_record_type);
CREATE UNIQUE INDEX idx_source_record_unique ON source_record (source, source_record_type, source_record_id);

CREATE TABLE evidence (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_record_id UUID NOT NULL REFERENCES source_record(id) ON DELETE CASCADE,
    quote_or_field_path TEXT,
    source_url TEXT,
    retrieved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    confidence DOUBLE PRECISION
);

CREATE INDEX idx_evidence_source_record ON evidence (source_record_id);

CREATE TABLE relationship_edge (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    edge_type edge_type NOT NULL,
    started_on DATE,
    ended_on DATE,
    amount DOUBLE PRECISION,
    currency TEXT DEFAULT 'USD',
    description TEXT,
    confidence DOUBLE PRECISION,
    evidence_id UUID REFERENCES evidence(id),
    source TEXT NOT NULL
);

CREATE INDEX idx_relationship_edge_from ON relationship_edge (from_entity_id);
CREATE INDEX idx_relationship_edge_to ON relationship_edge (to_entity_id);
CREATE INDEX idx_relationship_edge_type ON relationship_edge (edge_type);

CREATE TABLE award (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    generated_unique_award_id TEXT NOT NULL,
    recipient_entity_id UUID REFERENCES entity(id),
    awarding_agency_entity_id UUID REFERENCES entity(id),
    funding_agency_entity_id UUID REFERENCES entity(id),
    award_type TEXT,
    description TEXT,
    period_start DATE,
    period_end DATE,
    obligation_amount DOUBLE PRECISION,
    outlay_amount DOUBLE PRECISION,
    place_of_performance TEXT,
    evidence_id UUID REFERENCES evidence(id)
);

CREATE UNIQUE INDEX idx_award_unique_id ON award (generated_unique_award_id);
CREATE INDEX idx_award_recipient ON award (recipient_entity_id);
CREATE INDEX idx_award_period ON award (period_start, period_end);

CREATE TABLE lobbying_filing (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filing_uuid TEXT NOT NULL,
    registrant_entity_id UUID REFERENCES entity(id),
    client_entity_id UUID REFERENCES entity(id),
    filing_type TEXT,
    filing_period TEXT,
    filing_year INTEGER,
    income_or_expense TEXT,
    amount DOUBLE PRECISION,
    issues_json JSONB,
    evidence_id UUID REFERENCES evidence(id)
);

CREATE UNIQUE INDEX idx_lobbying_filing_uuid ON lobbying_filing (filing_uuid);
CREATE INDEX idx_lobbying_filing_registrant ON lobbying_filing (registrant_entity_id);
CREATE INDEX idx_lobbying_filing_client ON lobbying_filing (client_entity_id);

CREATE TABLE campaign_finance_transaction (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_id TEXT NOT NULL,
    committee_entity_id UUID REFERENCES entity(id),
    contributor_entity_id UUID REFERENCES entity(id),
    candidate_entity_id UUID REFERENCES entity(id),
    amount DOUBLE PRECISION,
    date DATE,
    employer_text TEXT,
    occupation_text TEXT,
    transaction_type TEXT,
    evidence_id UUID REFERENCES evidence(id)
);

CREATE UNIQUE INDEX idx_cf_transaction_id ON campaign_finance_transaction (transaction_id);
CREATE INDEX idx_cf_transaction_committee ON campaign_finance_transaction (committee_entity_id);
CREATE INDEX idx_cf_transaction_date ON campaign_finance_transaction (date);

CREATE TABLE sec_fact (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cik TEXT NOT NULL,
    company_entity_id UUID REFERENCES entity(id),
    accession_number TEXT NOT NULL,
    form TEXT,
    filed_at DATE,
    taxonomy TEXT,
    fact_name TEXT,
    unit TEXT,
    value DOUBLE PRECISION,
    period_start DATE,
    period_end DATE,
    frame TEXT,
    evidence_id UUID REFERENCES evidence(id)
);

CREATE INDEX idx_sec_fact_cik ON sec_fact (cik);
CREATE INDEX idx_sec_fact_company ON sec_fact (company_entity_id);
CREATE INDEX idx_sec_fact_fact_name ON sec_fact (fact_name);

CREATE TABLE rulemaking_record (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id TEXT,
    docket_id TEXT,
    agency_entity_id UUID REFERENCES entity(id),
    title TEXT,
    document_type TEXT,
    publication_date DATE,
    comment_start_date DATE,
    comment_end_date DATE,
    source_url TEXT,
    evidence_id UUID REFERENCES evidence(id)
);

CREATE INDEX idx_rulemaking_agency ON rulemaking_record (agency_entity_id);
CREATE INDEX idx_rulemaking_publication_date ON rulemaking_record (publication_date);
CREATE UNIQUE INDEX idx_rulemaking_document_id ON rulemaking_record (document_id) WHERE document_id IS NOT NULL;
