use std::fs;
use std::path::Path;

use crate::error::Result;

const INDEX_TS_TEMPLATE: &str = include_str!("../../templates/cloudflare/index.ts.tmpl");
const WRANGLER_TOML_TEMPLATE: &str = include_str!("../../templates/cloudflare/wrangler.toml.tmpl");
const SCHEMA_SQL: &str = include_str!("../../templates/cloudflare/schema.sql");
const PACKAGE_JSON_TEMPLATE: &str = include_str!("../../templates/cloudflare/package.json.tmpl");
const TSCONFIG_JSON: &str = include_str!("../../templates/cloudflare/tsconfig.json");

pub struct DeployOptions {
    pub dir: Option<String>,
    pub json: bool,
}

pub fn run(options: DeployOptions) -> Result<()> {
    let output_dir = options.dir.unwrap_or_else(|| "spikes-worker".to_string());
    let output_path = Path::new(&output_dir);

    // Check if directory exists
    if output_path.exists() {
        if options.json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": format!("Directory '{}' already exists", output_dir)
                })
            );
        } else {
            eprintln!("Error: Directory '{}' already exists", output_dir);
            eprintln!("Remove it or specify a different directory with --dir");
        }
        return Ok(());
    }

    // Generate token
    let token = generate_token();
    let project_name = sanitize_project_name(&output_dir);
    let db_name = format!("{}-db", project_name);

    // Create directory structure
    fs::create_dir_all(output_path.join("src"))?;

    // Write index.ts (no templating needed)
    fs::write(output_path.join("src/index.ts"), INDEX_TS_TEMPLATE)?;

    // Write wrangler.toml with placeholders
    let wrangler_toml = WRANGLER_TOML_TEMPLATE
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{TOKEN}}", &token)
        .replace("{{DB_NAME}}", &db_name)
        .replace("{{DB_ID}}", "<YOUR_D1_DATABASE_ID>");

    fs::write(output_path.join("wrangler.toml"), wrangler_toml)?;

    // Write schema.sql
    fs::write(output_path.join("schema.sql"), SCHEMA_SQL)?;

    // Write package.json
    let package_json = PACKAGE_JSON_TEMPLATE
        .replace("{{PROJECT_NAME}}", &project_name)
        .replace("{{DB_NAME}}", &db_name);

    fs::write(output_path.join("package.json"), package_json)?;

    // Write tsconfig.json
    fs::write(output_path.join("tsconfig.json"), TSCONFIG_JSON)?;

    // Write README
    let readme = generate_readme(&project_name, &db_name, &token);
    fs::write(output_path.join("README.md"), readme)?;

    // Save config to .spikes/config.toml if it exists
    let spikes_dir = Path::new(".spikes");
    if spikes_dir.exists() {
        let config_path = spikes_dir.join("config.toml");
        let config_content = format!(
            r#"# Spikes configuration
# https://spikes.sh

[project]
# Project key for grouping spikes
# key = "my-project"

[remote]
# Cloudflare Worker endpoint (update after deploying)
# endpoint = "https://{}.YOUR_SUBDOMAIN.workers.dev"
token = "{}"
"#,
            project_name, token
        );
        fs::write(config_path, config_content)?;
    }

    if options.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "directory": output_dir,
                "project_name": project_name,
                "db_name": db_name,
                "token": token,
                "files": [
                    format!("{}/src/index.ts", output_dir),
                    format!("{}/wrangler.toml", output_dir),
                    format!("{}/schema.sql", output_dir),
                    format!("{}/package.json", output_dir),
                    format!("{}/tsconfig.json", output_dir),
                    format!("{}/README.md", output_dir),
                ],
                "next_steps": [
                    format!("cd {}", output_dir),
                    "npm install",
                    format!("wrangler d1 create {}", db_name),
                    "# Update wrangler.toml with the database_id from above",
                    format!("wrangler d1 execute {} --file=schema.sql", db_name),
                    "wrangler deploy"
                ]
            })
        );
    } else {
        println!();
        println!("  🗡️  Spikes Cloudflare Worker scaffolded!");
        println!();
        println!("  Directory:    {}", output_dir);
        println!("  Auth Token:   {}", token);
        println!();
        println!("  Next steps:");
        println!();
        println!("    1. cd {}", output_dir);
        println!("    2. npm install");
        println!("    3. wrangler d1 create {}", db_name);
        println!("    4. Update wrangler.toml with the database_id from above");
        println!("    5. wrangler d1 execute {} --file=schema.sql", db_name);
        println!("    6. wrangler deploy");
        println!();
        println!("  After deploying, add to your widget:");
        println!();
        println!(
            r#"    <script src="spikes.js" data-endpoint="https://{}.YOUR_SUBDOMAIN.workers.dev/spikes?token={}"></script>"#,
            project_name, token
        );
        println!();
        println!("  Or configure in .spikes/config.toml for CLI sync:");
        println!();
        println!("    [remote]");
        println!(
            r#"    endpoint = "https://{}.YOUR_SUBDOMAIN.workers.dev""#,
            project_name
        );
        println!(r#"    token = "{}""#, token);
        println!();
    }

    Ok(())
}

fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Simple token generation using timestamp and random-ish values
    let mut hash: u64 = timestamp as u64;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;

    let hash2 = hash.wrapping_mul(0x9e3779b97f4a7c15);

    format!("{:016x}-{:016x}", hash, hash2)
}

fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn generate_readme(project_name: &str, db_name: &str, token: &str) -> String {
    format!(
        r#"# {} — Spikes Worker

Cloudflare Worker + D1 backend for multi-reviewer feedback sync.

Generated by: `spikes deploy cloudflare`

## Setup

1. Install dependencies:

```bash
npm install
```

2. Create the D1 database:

```bash
wrangler d1 create {}
```

3. Copy the `database_id` from the output and update `wrangler.toml`:

```toml
[[d1_databases]]
binding = "DB"
database_name = "{}"
database_id = "YOUR_DATABASE_ID_HERE"
```

4. Run the migration:

```bash
wrangler d1 execute {} --file=schema.sql
```

5. Deploy:

```bash
wrangler deploy
```

## Usage

### Widget Configuration

Add the `data-endpoint` attribute to your widget script tag:

```html
<script 
  src="spikes.js" 
  data-endpoint="https://{}.YOUR_SUBDOMAIN.workers.dev/spikes?token={}"
></script>
```

### CLI Configuration

Add to `.spikes/config.toml`:

```toml
[remote]
endpoint = "https://{}.YOUR_SUBDOMAIN.workers.dev"
token = "{}"
```

Then use:

```bash
spikes pull   # Fetch remote spikes to local
spikes push   # Upload local spikes to remote
```

## API

All endpoints require `?token={}` query parameter.

### POST /spikes

Create a new spike.

```bash
curl -X POST "https://YOUR_WORKER/spikes?token={}" \
  -H "Content-Type: application/json" \
  -d '{{"id":"abc","page":"Homepage","comments":"Great!"}}'
```

### GET /spikes

List all spikes. Supports filters:

- `?page=Homepage` — filter by page name
- `?reviewer=Patricia` — filter by reviewer name
- `?rating=no` — filter by rating
- `?project=my-project` — filter by project

```bash
curl "https://YOUR_WORKER/spikes?token={}"
```

### GET /spikes/:id

Get a single spike by ID.

```bash
curl "https://YOUR_WORKER/spikes/abc123?token={}"
```

## Auth Token

Your auth token: `{}`

Keep this secret! Anyone with the token can read/write spikes.

## Local Development

```bash
npm run dev
```

This starts a local development server with a local D1 database.
"#,
        project_name,
        db_name,
        db_name,
        db_name,
        project_name,
        token,
        project_name,
        token,
        token,
        token,
        token,
        token,
        token
    )
}
