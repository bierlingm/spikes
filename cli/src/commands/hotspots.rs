use std::collections::HashMap;

use crate::error::Result;
use crate::output::{print_hotspots_table, print_json};
use crate::spike::SpikeType;
use crate::storage::load_spikes;

pub fn run(json: bool) -> Result<()> {
    let spikes = load_spikes()?;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for spike in spikes {
        if spike.spike_type == SpikeType::Element {
            if let Some(selector) = spike.selector {
                *counts.entry(selector).or_insert(0) += 1;
            }
        }
    }

    let mut hotspots: Vec<(String, usize)> = counts.into_iter().collect();
    hotspots.sort_by(|a, b| b.1.cmp(&a.1));

    if json {
        let output: Vec<serde_json::Value> = hotspots
            .iter()
            .map(|(selector, count)| {
                serde_json::json!({
                    "selector": selector,
                    "count": count
                })
            })
            .collect();
        print_json(&output);
    } else {
        print_hotspots_table(&hotspots);
    }

    Ok(())
}
