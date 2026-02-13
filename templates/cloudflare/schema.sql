-- Spikes Self-Host Schema
-- Run: wrangler d1 execute <database-name> --file=schema.sql

CREATE TABLE IF NOT EXISTS shares (
    id TEXT PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    owner_token TEXT NOT NULL,
    created_at TEXT NOT NULL,
    spike_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_shares_slug ON shares(slug);
CREATE INDEX IF NOT EXISTS idx_shares_owner ON shares(owner_token);

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
    reviewer_email TEXT,
    timestamp TEXT NOT NULL,
    viewport TEXT,
    user_agent TEXT,
    share_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_spikes_project ON spikes(project);
CREATE INDEX IF NOT EXISTS idx_spikes_share ON spikes(share_id);
CREATE INDEX IF NOT EXISTS idx_spikes_timestamp ON spikes(timestamp);
