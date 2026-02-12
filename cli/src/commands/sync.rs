use crate::config::Config;
use crate::error::Result;

use super::pull::PullOptions;
use super::push::PushOptions;

/// Sync with remote: pull then push
pub fn run(json: bool) -> Result<()> {
    let config = Config::load()?;

    if config.effective_endpoint().is_none() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "success": false,
                    "error": "No remote configured. Use: spikes remote add <url>"
                })
            );
        } else {
            eprintln!();
            eprintln!("  / No remote configured");
            eprintln!();
            eprintln!("  Add a remote with:");
            eprintln!("    spikes remote add https://your-worker.workers.dev --token <token>");
            eprintln!();
            eprintln!("  Or deploy your own:");
            eprintln!("    spikes deploy cloudflare");
            eprintln!();
        }
        return Ok(());
    }

    if !json {
        println!();
        println!("  / Syncing with remote...");
        println!();
    }

    // Pull first
    let pull_result = super::pull::run(PullOptions {
        endpoint: config.remote.endpoint.clone(),
        token: config.remote.token.clone(),
        json,
    });

    if let Err(e) = pull_result {
        if !json {
            eprintln!("  Pull failed: {}", e);
        }
        return Err(e);
    }

    // Then push
    let push_result = super::push::run(PushOptions {
        endpoint: config.remote.endpoint.clone(),
        token: config.remote.token.clone(),
        json,
    });

    if let Err(e) = push_result {
        if !json {
            eprintln!("  Push failed: {}", e);
        }
        return Err(e);
    }

    if !json {
        println!();
        println!("  / Sync complete");
        println!();
    }

    Ok(())
}
