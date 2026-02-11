use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No .spikes/ directory found. Run 'spikes init' first.")]
    NoSpikesDir,

    #[error("Spike not found: {0}")]
    SpikeNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
