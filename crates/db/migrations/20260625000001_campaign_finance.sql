ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'candidate';
ALTER TYPE identifier_type ADD VALUE IF NOT EXISTS 'fec_candidate_id';
ALTER TYPE edge_type ADD VALUE IF NOT EXISTS 'committee_contributed_to_candidate';
ALTER TYPE edge_type ADD VALUE IF NOT EXISTS 'committee_transferred_to_committee';
ALTER TYPE edge_type ADD VALUE IF NOT EXISTS 'candidate_supported_by_committee';

ALTER TABLE campaign_finance_transaction
  ADD COLUMN IF NOT EXISTS contributor_city TEXT,
  ADD COLUMN IF NOT EXISTS contributor_state TEXT,
  ADD COLUMN IF NOT EXISTS contributor_zip TEXT,
  ADD COLUMN IF NOT EXISTS committee_fec_id TEXT,
  ADD COLUMN IF NOT EXISTS candidate_fec_id TEXT,
  ADD COLUMN IF NOT EXISTS memo_text TEXT,
  ADD COLUMN IF NOT EXISTS transaction_type_code TEXT,
  ADD COLUMN IF NOT EXISTS cycle INT,
  ADD COLUMN IF NOT EXISTS recipient_committee_name TEXT,
  ADD COLUMN IF NOT EXISTS other_entity_id UUID REFERENCES entity(id),
  ADD COLUMN IF NOT EXISTS sub_id TEXT;

CREATE INDEX IF NOT EXISTS idx_cf_txn_contributor ON campaign_finance_transaction (contributor_entity_id);
CREATE INDEX IF NOT EXISTS idx_cf_txn_committee ON campaign_finance_transaction (committee_entity_id);
CREATE INDEX IF NOT EXISTS idx_cf_txn_committee_fec ON campaign_finance_transaction (committee_fec_id);
CREATE INDEX IF NOT EXISTS idx_cf_txn_amount ON campaign_finance_transaction (amount DESC);
CREATE INDEX IF NOT EXISTS idx_cf_txn_cycle ON campaign_finance_transaction (cycle);
CREATE INDEX IF NOT EXISTS idx_cf_txn_type ON campaign_finance_transaction (transaction_type);
CREATE INDEX IF NOT EXISTS idx_cf_txn_sub_id ON campaign_finance_transaction (sub_id);
CREATE INDEX IF NOT EXISTS idx_cf_txn_date ON campaign_finance_transaction (date);

CREATE TABLE candidate_metadata (
  entity_id UUID PRIMARY KEY REFERENCES entity(id),
  candidate_fec_id TEXT NOT NULL,
  party TEXT,
  office TEXT,
  office_state TEXT,
  office_district TEXT,
  incumbent_challenge TEXT,
  election_year INT,
  candidate_status TEXT,
  principal_committee_fec_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_candidate_metadata_fec_id ON candidate_metadata (candidate_fec_id);

CREATE TABLE committee_metadata (
  entity_id UUID PRIMARY KEY REFERENCES entity(id),
  committee_fec_id TEXT NOT NULL,
  committee_type TEXT,
  committee_designation TEXT,
  party TEXT,
  treasurer_name TEXT,
  organization_type TEXT,
  connected_organization TEXT
);
CREATE INDEX IF NOT EXISTS idx_committee_metadata_fec_id ON committee_metadata (committee_fec_id);

CREATE TABLE candidate_committee_link (
  candidate_entity_id UUID REFERENCES entity(id),
  committee_entity_id UUID REFERENCES entity(id),
  candidate_fec_id TEXT,
  committee_fec_id TEXT,
  committee_designation TEXT,
  PRIMARY KEY (candidate_fec_id, committee_fec_id)
);
