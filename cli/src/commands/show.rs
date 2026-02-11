use crate::error::{Error, Result};
use crate::output::{print_json, print_spike_detail};
use crate::storage::load_spikes;

pub fn run(id: &str, json: bool) -> Result<()> {
    let spikes = load_spikes()?;

    let spike = spikes
        .into_iter()
        .find(|s| s.id == id || s.id.starts_with(id))
        .ok_or_else(|| Error::SpikeNotFound(id.to_string()))?;

    if json {
        print_json(&spike);
    } else {
        print_spike_detail(&spike);
    }

    Ok(())
}
