use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::config::Config;
use crate::error::Result;

const INDEX_TS_TEMPLATE: &str = include_str!("../../templates/cloudflare/index.ts.tmpl");
const WRANGLER_TOML_TEMPLATE: &str = include_str!("../../templates/cloudflare/wrangler.toml.tmpl");
const SCHEMA_SQL: &str = include_str!("../../templates/cloudflare/schema.sql");
const PACKAGE_JSON_TEMPLATE: &str = include_str!("../../templates/cloudflare/package.json.tmpl");
const TSCONFIG_JSON: &str = include_str!("../../templates/cloudflare/tsconfig.json");

pub struct DeployOptions {
    pub dir: Option<String>,
    pub json: bool,
    pub force: bool,
}

/// Check if the project is configured for hosted spikes.sh
fn is_hosted_config() -> bool {
    match Config::load() {
        Ok(config) => config.remote.hosted,
        Err(_) => false,
    }
}

/// Check if stdin is a TTY (interactive terminal)
fn is_interactive() -> bool {
    io::stdin().is_terminal()
}

/// Print the hosted warning and prompt for confirmation
/// Returns Ok(true) to proceed, Ok(false) to abort (caller should exit with appropriate code)
fn prompt_hosted_warning(json: bool, force: bool) -> Result<bool> {
    // If force flag is set, skip the warning
    if force {
        return Ok(true);
    }

    // In non-interactive mode (no TTY), print warning to stderr and abort non-zero
    if !is_interactive() {
        if json {
            eprintln!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": "Hosted backend detected; use --force to override",
                    "code": "HOSTED_BACKEND"
                })
            );
        } else {
            eprintln!("Warning: This project is configured for spikes.sh hosted backend.");
            eprintln!("Deploy cloudflare is for data isolation or custom domain use cases.");
            eprintln!("Use --force to deploy Cloudflare Worker anyway.");
        }
        // Return false to abort - caller should exit non-zero
        return Ok(false);
    }

    // Interactive mode - print warning and prompt
    println!();
    println!("⚠️  Warning: This project is configured for spikes.sh hosted backend.");
    println!("   spikes.sh already hosts this backend for you.");
    println!();
    println!("   Use `spikes deploy cloudflare` only if you need:");
    println!("   • Data isolation (keep feedback data in your own Cloudflare account)");
    println!("   • Custom domain for your feedback API");
    println!();
    print!("Continue? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let trimmed = input.trim();

    // Empty (Enter), 'n', or 'N' = abort
    // 'y' or 'Y' = proceed
    match trimmed {
        "y" | "Y" => Ok(true),
        _ => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": "Aborted: deploy cancelled by user"
                    })
                );
            } else {
                println!("Aborted: deploy cancelled.");
            }
            Ok(false)
        }
    }
}

/// Check if a directory is empty (contains no files/directories)
/// ANY directory entry, including hidden dotfiles (e.g., .git, .spikes, .env),
/// causes the directory to be treated as non-empty.
fn is_directory_empty(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    // Check if there are ANY entries - hidden dotfiles count as non-empty
    // This prevents scaffolding into directories with .git/, .spikes/, .env, etc.
    let mut entries = fs::read_dir(path)?;
    if entries.next().is_some() {
        return Ok(false);
    }

    Ok(true)
}

pub fn run(options: DeployOptions) -> Result<()> {
    let output_dir = options.dir.unwrap_or_else(|| "spikes-worker".to_string());
    let output_path = Path::new(&output_dir);

    // Check if directory exists and is non-empty
    if output_path.exists() && !is_directory_empty(output_path)? {
        if options.json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": format!("Directory '{}' is not empty", output_dir)
                })
            );
        } else {
            eprintln!("Error: Directory '{}' is not empty", output_dir);
            eprintln!("Remove existing files or specify a different directory with --dir");
        }
        // Return Err instead of Ok to ensure non-zero exit
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Directory '{}' is not empty", output_dir),
        )));
    }

    // Check if this is a hosted config and show warning/prompt
    if is_hosted_config() {
        match prompt_hosted_warning(options.json, options.force) {
            Ok(false) => {
                // User chose not to proceed (or non-interactive abort)
                // Non-interactive mode always exits non-zero (1)
                if !is_interactive() {
                    std::process::exit(1);
                }
                // Interactive mode: JSON output already printed by prompt_hosted_warning, exit 0
                // (human mode also printed "Aborted" message)
                std::process::exit(0);
            }
            Ok(true) => {
                // Proceed with deploy
            }
            Err(e) => return Err(e),
        }
    }

    // Generate token
    let token = generate_token();
    let project_name = sanitize_project_name(&output_dir);
    let db_name = format!("{}-db", project_name);

    // Create directory structure (handles both new dirs and empty existing dirs)
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

    // Save config to .spikes/config.toml if it exists (merge instead of replace)
    let spikes_dir = Path::new(".spikes");
    if spikes_dir.exists() {
        update_config_with_token(&token, &project_name, options.json)?;
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
            r#"    <script src="widget.js" data-endpoint="https://{}.YOUR_SUBDOMAIN.workers.dev/spikes?token={}"></script>"#,
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

/// Update the existing config.toml with new remote token, preserving other sections
fn update_config_with_token(token: &str, _project_name: &str, _json: bool) -> Result<()> {
    // Load existing config
    let config = Config::load()?;

    // Create updated config with new remote settings
    let mut new_config = config.clone();

    // Only update the remote section - preserve other sections
    new_config.remote.token = Some(token.to_string());
    // Note: we don't change hosted status here - if user was hosted, they remain hosted
    // but with a token for the new self-hosted backend
    new_config.remote.hosted = false; // Self-hosted backend is not "hosted"

    // Save the merged config
    new_config.save()?;

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

Self-hosted Cloudflare Worker + D1 backend for multi-reviewer feedback sync.

Generated by: `spikes deploy cloudflare`

## Why Self-Host?

**spikes.sh already hosts this backend** — for most users, the hosted service is the best choice:
- No setup or maintenance required
- Automatic updates and scaling
- Built-in security and backups

Use `spikes deploy cloudflare` only if you need:
- **Data isolation** — Keep all feedback data within your own Cloudflare account
- **Custom domain** — Serve your feedback API from your own domain (e.g., `api.yourdomain.com`)
- **Enterprise compliance** — Meet organizational requirements for data residency

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
  src="widget.js"
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_token_format() {
        let token = generate_token();
        // Token should match pattern: 16 hex chars - 16 hex chars
        assert!(token.len() == 33, "Token should be 33 characters (16 + 1 + 16)");
        assert!(token.contains('-'), "Token should contain hyphen separator");
        
        let parts: Vec<&str> = token.split('-').collect();
        assert_eq!(parts.len(), 2, "Token should have exactly 2 parts");
        assert_eq!(parts[0].len(), 16, "First part should be 16 hex chars");
        assert_eq!(parts[1].len(), 16, "Second part should be 16 hex chars");
        
        // Verify hex format
        assert!(u64::from_str_radix(parts[0], 16).is_ok(), "First part should be valid hex");
        assert!(u64::from_str_radix(parts[1], 16).is_ok(), "Second part should be valid hex");
    }

    #[test]
    fn test_sanitize_project_name() {
        assert_eq!(sanitize_project_name("My Project"), "my-project");
        assert_eq!(sanitize_project_name("test_project"), "test-project");
        assert_eq!(sanitize_project_name("UPPERCASE"), "uppercase");
        assert_eq!(sanitize_project_name("---leading-dashes"), "leading-dashes");
        assert_eq!(sanitize_project_name("trailing-dashes---"), "trailing-dashes");
        assert_eq!(sanitize_project_name("a-b-c-123"), "a-b-c-123");
    }

    #[test]
    fn test_generate_readme_contains_hosted_info() {
        let readme = generate_readme("test-project", "test-project-db", "test-token-1234");
        
        // Check for spikes.sh mention
        assert!(readme.contains("spikes.sh"), "README should mention spikes.sh");
        
        // Check for data isolation mention
        assert!(
            readme.to_lowercase().contains("data isolation") || 
            readme.to_lowercase().contains("custom domain") ||
            readme.to_lowercase().contains("self-host"),
            "README should mention data isolation, custom domain, or self-host rationale"
        );
        
        // Check for "Why Self-Host" section
        assert!(readme.contains("Why Self-Host"), "README should have 'Why Self-Host' section");
    }

    #[test]
    fn test_generate_readme_contains_token() {
        let token = "abcd1234-5678efgh";
        let readme = generate_readme("test", "test-db", token);
        
        // Token should appear in README
        assert!(readme.contains(token), "README should contain the auth token");
    }

    #[test]
    fn test_is_directory_empty_new_dir() {
        let temp_dir = TempDir::new().unwrap();
        let new_path = temp_dir.path().join("new_dir");
        
        // Non-existent directory should be considered "empty" (will be created)
        assert!(is_directory_empty(&new_path).unwrap(), "Non-existent dir should be considered empty");
    }

    #[test]
    fn test_is_directory_empty_actually_empty() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();
        
        assert!(is_directory_empty(&empty_dir).unwrap(), "Truly empty dir should be empty");
    }

    #[test]
    fn test_is_directory_empty_with_hidden_files() {
        let temp_dir = TempDir::new().unwrap();
        let hidden_dir = temp_dir.path().join("hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join(".gitkeep"), "").unwrap();
        
        // Hidden files DO count as non-empty (regression fix: ANY entry counts)
        assert!(!is_directory_empty(&hidden_dir).unwrap(), "Dir with hidden files should be considered non-empty");
    }

    #[test]
    fn test_is_directory_empty_with_dot_git() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join("git_project");
        fs::create_dir(&git_dir).unwrap();
        fs::create_dir(git_dir.join(".git")).unwrap();
        
        // .git/ directory counts as non-empty
        assert!(!is_directory_empty(&git_dir).unwrap(), "Dir with .git/ should be considered non-empty");
    }

    #[test]
    fn test_is_directory_empty_with_dot_spikes() {
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join("spikes_project");
        fs::create_dir(&spikes_dir).unwrap();
        fs::create_dir(spikes_dir.join(".spikes")).unwrap();
        
        // .spikes/ directory counts as non-empty
        assert!(!is_directory_empty(&spikes_dir).unwrap(), "Dir with .spikes/ should be considered non-empty");
    }

    #[test]
    fn test_is_directory_empty_with_dot_env() {
        let temp_dir = TempDir::new().unwrap();
        let env_dir = temp_dir.path().join("env_project");
        fs::create_dir(&env_dir).unwrap();
        fs::write(env_dir.join(".env"), "SECRET=value").unwrap();
        
        // .env file counts as non-empty
        assert!(!is_directory_empty(&env_dir).unwrap(), "Dir with .env file should be considered non-empty");
    }

    #[test]
    fn test_is_directory_empty_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_empty_dir = temp_dir.path().join("nonempty");
        fs::create_dir(&non_empty_dir).unwrap();
        fs::write(non_empty_dir.join("some-file.txt"), "content").unwrap();
        
        assert!(!is_directory_empty(&non_empty_dir).unwrap(), "Dir with files should not be empty");
    }

    #[test]
    fn test_is_directory_empty_with_visible_dir() {
        let temp_dir = TempDir::new().unwrap();
        let parent_dir = temp_dir.path().join("parent");
        fs::create_dir(&parent_dir).unwrap();
        fs::create_dir(parent_dir.join("subdir")).unwrap();
        
        assert!(!is_directory_empty(&parent_dir).unwrap(), "Dir with subdirectories should not be empty");
    }
}
