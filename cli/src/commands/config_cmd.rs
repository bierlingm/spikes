use crate::config::Config;
use crate::error::Result;

/// Show current configuration
pub fn run(json: bool) -> Result<()> {
    let config = Config::load()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "project": {
                    "key": config.effective_project_key()
                },
                "widget": {
                    "theme": config.widget.theme,
                    "position": config.widget.position,
                    "color": config.widget.color,
                    "collect_email": config.widget.collect_email
                },
                "remote": {
                    "endpoint": config.effective_endpoint(),
                    "hosted": config.remote.hosted,
                    "has_token": config.remote.token.is_some()
                }
            })
        );
    } else {
        println!();
        println!("  / Spikes Configuration");
        println!();
        println!("  Project:  {}", config.effective_project_key());
        println!();
        println!("  Widget:");
        println!("    theme:         {}", config.widget.theme);
        println!("    position:      {}", config.widget.position);
        println!("    color:         {}", config.widget.color);
        println!("    collect_email: {}", config.widget.collect_email);
        println!();
        println!("  Remote:");
        if let Some(endpoint) = config.effective_endpoint() {
            println!("    endpoint: {}", endpoint);
            println!("    hosted:   {}", config.remote.hosted);
            println!("    token:    {}", if config.remote.token.is_some() { "(set)" } else { "(not set)" });
        } else {
            println!("    (not configured)");
        }
        println!();
        println!("  Widget tag attributes:");
        println!("    {}", config.widget_attributes());
        println!();
    }

    Ok(())
}
