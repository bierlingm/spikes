use std::collections::HashMap;

use crate::error::Result;
use crate::output::{print_json, print_reviewers_table};
use crate::storage::load_spikes;

pub fn run(json: bool) -> Result<()> {
    let spikes = load_spikes()?;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for spike in spikes {
        *counts.entry(spike.reviewer.name).or_insert(0) += 1;
    }

    let mut reviewers: Vec<(String, usize)> = counts.into_iter().collect();
    reviewers.sort_by(|a, b| b.1.cmp(&a.1));

    if json {
        let output: Vec<serde_json::Value> = reviewers
            .iter()
            .map(|(name, count)| {
                serde_json::json!({
                    "name": name,
                    "count": count
                })
            })
            .collect();
        print_json(&output);
    } else {
        print_reviewers_table(&reviewers);
    }

    Ok(())
}
