CREATE TABLE geo (
    geo_id TEXT PRIMARY KEY,
    geo_type TEXT NOT NULL,
    name TEXT NOT NULL,
    state_code TEXT,
    county_code TEXT,
    region TEXT
);

CREATE INDEX idx_geo_type ON geo (geo_type);
CREATE INDEX idx_geo_state_code ON geo (state_code);

CREATE TABLE data_source (
    source_id TEXT PRIMARY KEY,
    agency TEXT NOT NULL,
    dataset TEXT NOT NULL,
    url TEXT NOT NULL,
    license TEXT,
    update_frequency TEXT,
    requires_api_key BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT
);

CREATE TABLE metric (
    metric_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    unit TEXT NOT NULL,
    seasonal_adjustment TEXT,
    source_id TEXT NOT NULL REFERENCES data_source(source_id),
    notes TEXT
);

CREATE INDEX idx_metric_category ON metric (category);
CREATE INDEX idx_metric_source ON metric (source_id);

CREATE TABLE metric_observation (
    metric_id TEXT NOT NULL REFERENCES metric(metric_id),
    geo_id TEXT NOT NULL REFERENCES geo(geo_id),
    date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    vintage_date DATE NOT NULL,
    release_date DATE,
    source_series_id TEXT,
    evidence_id UUID REFERENCES evidence(id),
    notes TEXT,
    PRIMARY KEY(metric_id, geo_id, date, vintage_date)
);

CREATE INDEX idx_metric_observation_geo_date ON metric_observation (geo_id, date DESC);
CREATE INDEX idx_metric_observation_metric_date ON metric_observation (metric_id, date DESC);

CREATE TABLE derived_metric_observation (
    metric_id TEXT NOT NULL,
    geo_id TEXT NOT NULL REFERENCES geo(geo_id),
    date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    formula TEXT NOT NULL,
    input_metric_ids TEXT[] NOT NULL,
    vintage_date DATE NOT NULL,
    notes TEXT,
    PRIMARY KEY(metric_id, geo_id, date, vintage_date)
);

CREATE INDEX idx_derived_metric_observation_geo_date ON derived_metric_observation (geo_id, date DESC);

ALTER TABLE award
    ADD COLUMN recipient_geo_id TEXT REFERENCES geo(geo_id),
    ADD COLUMN place_geo_id TEXT REFERENCES geo(geo_id);

CREATE INDEX idx_award_recipient_geo ON award (recipient_geo_id);
CREATE INDEX idx_award_place_geo ON award (place_geo_id);
