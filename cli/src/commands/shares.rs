use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};
use serde::{Deserialize, Serialize};

use crate::auth::AuthConfig;
use crate::error::{map_http_error, map_network_error, Error, Result};

pub struct SharesOptions {
    pub json: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub spike_count: usize,
    pub created_at: String,
}

pub fn run(options: SharesOptions) -> Result<()> {
    let token = AuthConfig::token()?
        .ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Not logged in. Run 'spikes login' first.",
            ))
        })?;
    let shares = fetch_shares(&token)?;

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&shares).expect("Failed to serialize to JSON")
        );
    } else {
        print_shares_table(&shares);
    }

    Ok(())
}

fn fetch_shares(token: &str) -> Result<Vec<Share>> {
    let response = match ureq::get("https://spikes.sh/shares")
        .set("Authorization", &format!("Bearer {}", token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().ok();
            return Err(map_http_error(status, body.as_deref()));
        }
        Err(e) => return Err(map_network_error(&e.to_string())),
    };

    let status = response.status();

    if status != 200 {
        let body = response.into_string().ok();
        return Err(map_http_error(status, body.as_deref()));
    }

    let body = response
        .into_string()
        .map_err(|e| Error::RequestFailed(format!("Failed to read response: {}", e)))?;

    let shares: Vec<Share> = serde_json::from_str(&body)?;
    Ok(shares)
}

fn print_shares_table(shares: &[Share]) {
    if shares.is_empty() {
        println!("No shares found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Slug", "URL", "Spikes", "Created"]);

    for share in shares {
        table.add_row(vec![
            Cell::new(&share.slug),
            Cell::new(&share.url),
            Cell::new(share.spike_count),
            Cell::new(&share.created_at),
        ]);
    }

    println!("{table}");
}
