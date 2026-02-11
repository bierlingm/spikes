-- Spikes D1 Schema
-- Run: wrangler d1 execute <db-name> --file=schema.sql

CREATE TABLE IF NOT EXISTS spikes (
    id TEXT PRIMARY KEY,
    project TEXT NOT NULL,
    page TEXT NOT NULL,
    url TEXT,
    type TEXT NOT NULL,
    selector TEXT,
    xpath TEXT,
    element_text TEXT,
    bounding_box TEXT,
    rating TEXT,
    comments TEXT,
    reviewer_id TEXT NOT NULL,
    reviewer_name TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    viewport TEXT,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_spikes_project ON spikes(project);
CREATE INDEX IF NOT EXISTS idx_spikes_page ON spikes(page);
CREATE INDEX IF NOT EXISTS idx_spikes_reviewer ON spikes(reviewer_id);
CREATE INDEX IF NOT EXISTS idx_spikes_timestamp ON spikes(timestamp);
