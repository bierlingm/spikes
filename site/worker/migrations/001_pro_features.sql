-- V9 Pro Features Migration
-- Add columns for Pro tier features

-- Users: add subdomain for Pro routing (unique enforced at app level)
ALTER TABLE users ADD COLUMN subdomain TEXT;
CREATE INDEX IF NOT EXISTS idx_users_subdomain ON users(subdomain);
CREATE INDEX IF NOT EXISTS idx_users_stripe ON users(stripe_customer_id);

-- Shares: add owner_id, password_hash, webhook_url for Pro features
ALTER TABLE shares ADD COLUMN owner_id TEXT;
ALTER TABLE shares ADD COLUMN password_hash TEXT;
ALTER TABLE shares ADD COLUMN webhook_url TEXT;
CREATE INDEX IF NOT EXISTS idx_shares_owner_id ON shares(owner_id);
