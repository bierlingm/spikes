use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{Error, Result};
use crate::spike::Spike;

const FEEDBACK_FILE: &str = ".spikes/feedback.jsonl";

pub fn load_spikes() -> Result<Vec<Spike>> {
    let path = Path::new(FEEDBACK_FILE);

    if !path.exists() {
        return Err(Error::NoSpikesDir);
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut spikes = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let spike: Spike = serde_json::from_str(&line)?;
        spikes.push(spike);
    }

    Ok(spikes)
}
