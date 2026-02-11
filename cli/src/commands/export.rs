use std::io::{self, Write};

use crate::error::Result;
use crate::storage::load_spikes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Jsonl,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "jsonl" => Ok(ExportFormat::Jsonl),
            _ => Err(format!("Invalid format: {}. Use json, csv, or jsonl", s)),
        }
    }
}

pub fn run(format: ExportFormat) -> Result<()> {
    let spikes = load_spikes()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&spikes)?;
            writeln!(handle, "{}", json)?;
        }
        ExportFormat::Jsonl => {
            for spike in &spikes {
                let json = serde_json::to_string(spike)?;
                writeln!(handle, "{}", json)?;
            }
        }
        ExportFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(handle);
            wtr.write_record([
                "id",
                "type",
                "project_key",
                "page",
                "url",
                "reviewer_id",
                "reviewer_name",
                "selector",
                "element_text",
                "rating",
                "comments",
                "timestamp",
                "viewport_width",
                "viewport_height",
            ])?;

            for spike in &spikes {
                wtr.write_record([
                    &spike.id,
                    spike.type_str(),
                    &spike.project_key,
                    &spike.page,
                    &spike.url,
                    &spike.reviewer.id,
                    &spike.reviewer.name,
                    spike.selector.as_deref().unwrap_or(""),
                    spike.element_text.as_deref().unwrap_or(""),
                    spike.rating_str(),
                    &spike.comments,
                    &spike.timestamp,
                    &spike.viewport.width.to_string(),
                    &spike.viewport.height.to_string(),
                ])?;
            }
            wtr.flush()?;
        }
    }

    Ok(())
}
