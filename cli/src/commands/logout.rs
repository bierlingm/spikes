//! Logout command - remove stored authentication token

use crate::auth::AuthConfig;
use crate::error::Result;

pub fn run(json: bool) -> Result<()> {
    // Check if user is logged in
    let has_token = AuthConfig::has_token();

    if !has_token {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "success": true,
                    "message": "Already logged out"
                })
            );
        } else {
            println!();
            println!("  Already logged out.");
            println!();
        }
        return Ok(());
    }

    // Delete the auth file
    AuthConfig::clear_token()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "message": "Logged out successfully"
            })
        );
    } else {
        println!();
        println!("  🗡️  Logged out successfully");
        println!();
    }

    Ok(())
}
