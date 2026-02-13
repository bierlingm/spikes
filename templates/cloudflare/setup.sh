#!/bin/bash
set -e

echo "🔧 Spikes Self-Host Setup"
echo "========================="
echo

# Check wrangler
if ! command -v wrangler &> /dev/null; then
    echo "❌ wrangler not found. Install with: npm install -g wrangler"
    exit 1
fi

echo "✓ wrangler found"

# Check login
if ! wrangler whoami &> /dev/null; then
    echo "❌ Not logged in. Run: wrangler login"
    exit 1
fi

echo "✓ wrangler authenticated"
echo

# Prompt for names
read -p "Database name [my-spikes-db]: " DB_NAME
DB_NAME=${DB_NAME:-my-spikes-db}

read -p "R2 bucket name [my-spikes-assets]: " BUCKET_NAME
BUCKET_NAME=${BUCKET_NAME:-my-spikes-assets}

read -p "Worker name [my-spikes-worker]: " WORKER_NAME
WORKER_NAME=${WORKER_NAME:-my-spikes-worker}

echo
echo "Creating D1 database: $DB_NAME"
DB_OUTPUT=$(wrangler d1 create "$DB_NAME" 2>&1) || true
echo "$DB_OUTPUT"

# Extract database_id
DB_ID=$(echo "$DB_OUTPUT" | grep -o 'database_id = "[^"]*"' | cut -d'"' -f2)
if [ -z "$DB_ID" ]; then
    echo "⚠️  Could not extract database_id. Check output above and update wrangler.toml manually."
    DB_ID="YOUR_DATABASE_ID_HERE"
fi

echo
echo "Creating R2 bucket: $BUCKET_NAME"
wrangler r2 bucket create "$BUCKET_NAME" 2>&1 || true

echo
echo "Generating wrangler.toml"
cat > wrangler.toml << EOF
name = "$WORKER_NAME"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_name = "$DB_NAME"
database_id = "$DB_ID"

[[r2_buckets]]
binding = "ASSETS"
bucket_name = "$BUCKET_NAME"
EOF

echo "✓ wrangler.toml created"

echo
echo "Running schema..."
wrangler d1 execute "$DB_NAME" --file=schema.sql

echo
echo "Deploying worker..."
wrangler deploy

echo
echo "✅ Setup complete!"
echo
echo "Your endpoint: https://$WORKER_NAME.<subdomain>.workers.dev"
echo "Configure CLI: echo 'https://$WORKER_NAME.<subdomain>.workers.dev' > .spikes/endpoint"
