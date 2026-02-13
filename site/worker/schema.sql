-- Spikes D1 Schema for spikes.sh
-- Run: wrangler d1 execute spikes-sh-db --file=schema.sql

-- V9: Users table for hosted shares
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT,
    tier TEXT DEFAULT 'free',
    stripe_customer_id TEXT,
    subdomain TEXT UNIQUE,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_subdomain ON users(subdomain);
CREATE INDEX IF NOT EXISTS idx_users_stripe ON users(stripe_customer_id);

-- V9: Shares table for hosted share links
CREATE TABLE IF NOT EXISTS shares (
    id TEXT PRIMARY KEY,
    slug TEXT UNIQUE NOT NULL,
    owner_token TEXT NOT NULL,
    owner_id TEXT,
    password_hash TEXT,
    webhook_url TEXT,
    created_at TEXT NOT NULL,
    spike_count INTEGER DEFAULT 0,
    tier TEXT DEFAULT 'free'
);

CREATE INDEX IF NOT EXISTS idx_shares_owner_id ON shares(owner_id);

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
CREATE INDEX IF NOT EXISTS idx_spikes_page ON spikes(page);
CREATE INDEX IF NOT EXISTS idx_spikes_reviewer ON spikes(reviewer_id);
CREATE INDEX IF NOT EXISTS idx_spikes_timestamp ON spikes(timestamp);
CREATE INDEX IF NOT EXISTS idx_spikes_email ON spikes(reviewer_email);
CREATE INDEX IF NOT EXISTS idx_spikes_share ON spikes(share_id);
