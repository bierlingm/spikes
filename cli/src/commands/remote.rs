use crate::config::{self, Config};
use crate::error::Result;

/// Add or update remote endpoint
pub fn add(endpoint: &str, token: Option<String>, hosted: bool) -> Result<()> {
    // Ensure .spikes exists
    config::ensure_initialized()?;

    let mut config = Config::load()?;

    if hosted {
        config.remote.hosted = true;
        config.remote.endpoint = None;
    } else {
        config.remote.hosted = false;
        config.remote.endpoint = Some(endpoint.to_string());
    }

    if let Some(t) = token {
        config.remote.token = Some(t);
    }

    config.save()?;

    println!();
    println!("  / Remote configured");
    println!();

    if hosted {
        println!("  Using:  spikes.sh hosted backend");
    } else {
        println!("  Endpoint: {}", endpoint);
    }

    if config.remote.token.is_some() {
        println!("  Token:    (set)");
    } else {
        println!("  Token:    (not set — add with --token)");
    }

    println!();
    println!("  Next: spikes sync");
    println!();

    Ok(())
}

/// Remove remote configuration
pub fn remove() -> Result<()> {
    let mut config = Config::load()?;

    config.remote.endpoint = None;
    config.remote.token = None;
    config.remote.hosted = false;

    config.save()?;

    println!();
    println!("  / Remote configuration removed");
    println!();

    Ok(())
}

/// Show current remote configuration
pub fn show(json: bool) -> Result<()> {
    let config = Config::load()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "endpoint": config.remote.endpoint,
                "token": config.remote.token.as_ref().map(|_| "(set)"),
                "hosted": config.remote.hosted,
                "effective_endpoint": config.effective_endpoint()
            })
        );
    } else {
        println!();
        if let Some(endpoint) = config.effective_endpoint() {
            println!("  / Remote: {}", endpoint);
            if config.remote.hosted {
                println!("  Type:     spikes.sh hosted");
            } else {
                println!("  Type:     self-hosted");
            }
            if config.remote.token.is_some() {
                println!("  Token:    (set)");
            } else {
                println!("  Token:    (not set)");
            }
        } else {
            println!("  / No remote configured");
            println!();
            println!("  Add with: spikes remote add <url> --token <token>");
        }
        println!();
    }

    Ok(())
}
