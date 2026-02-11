use crate::error::Result;
use crate::output::{print_json, print_spikes_table};
use crate::spike::{Rating, Spike};
use crate::storage::load_spikes;

pub struct ListOptions {
    pub json: bool,
    pub page: Option<String>,
    pub reviewer: Option<String>,
    pub rating: Option<String>,
}

pub fn run(options: ListOptions) -> Result<()> {
    let spikes = load_spikes()?;

    let filtered: Vec<Spike> = spikes
        .into_iter()
        .filter(|s| {
            if let Some(ref page) = options.page {
                if !s.page.to_lowercase().contains(&page.to_lowercase()) {
                    return false;
                }
            }
            if let Some(ref reviewer) = options.reviewer {
                if !s
                    .reviewer
                    .name
                    .to_lowercase()
                    .contains(&reviewer.to_lowercase())
                {
                    return false;
                }
            }
            if let Some(ref rating) = options.rating {
                if let Ok(r) = rating.parse::<Rating>() {
                    if s.rating.as_ref() != Some(&r) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        })
        .collect();

    if options.json {
        print_json(&filtered);
    } else {
        print_spikes_table(&filtered);
    }

    Ok(())
}
