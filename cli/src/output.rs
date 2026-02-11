use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, ContentArrangement, Table};

use crate::spike::{Rating, Spike};

pub fn print_spikes_table(spikes: &[Spike]) {
    if spikes.is_empty() {
        println!("No spikes found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Type", "Page", "Reviewer", "Rating", "Comments"]);

    for spike in spikes {
        let rating_cell = match &spike.rating {
            Some(Rating::Love) => Cell::new("love").fg(Color::Green),
            Some(Rating::Like) => Cell::new("like").fg(Color::Blue),
            Some(Rating::Meh) => Cell::new("meh").fg(Color::Yellow),
            Some(Rating::No) => Cell::new("no").fg(Color::Red),
            None => Cell::new("-"),
        };

        let comments = if spike.comments.len() > 40 {
            format!("{}...", &spike.comments[..37])
        } else {
            spike.comments.clone()
        };

        table.add_row(vec![
            Cell::new(&spike.id[..8.min(spike.id.len())]),
            Cell::new(spike.type_str()),
            Cell::new(&spike.page),
            Cell::new(&spike.reviewer.name),
            rating_cell,
            Cell::new(comments),
        ]);
    }

    println!("{table}");
}

pub fn print_spike_detail(spike: &Spike) {
    println!("ID:         {}", spike.id);
    println!("Type:       {}", spike.type_str());
    println!("Project:    {}", spike.project_key);
    println!("Page:       {}", spike.page);
    println!("URL:        {}", spike.url);
    println!("Reviewer:   {} ({})", spike.reviewer.name, spike.reviewer.id);
    println!("Rating:     {}", spike.rating_str());
    println!("Timestamp:  {}", spike.timestamp);
    println!(
        "Viewport:   {}x{}",
        spike.viewport.width, spike.viewport.height
    );

    if let Some(selector) = &spike.selector {
        println!("Selector:   {}", selector);
    }
    if let Some(text) = &spike.element_text {
        println!("Element:    {}", text);
    }
    if let Some(bb) = &spike.bounding_box {
        println!(
            "BoundingBox: ({}, {}) {}x{}",
            bb.x, bb.y, bb.width, bb.height
        );
    }

    println!();
    println!("Comments:");
    println!("  {}", spike.comments);
}

pub fn print_json<T: serde::Serialize>(data: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(data).expect("Failed to serialize to JSON")
    );
}

pub fn print_hotspots_table(hotspots: &[(String, usize)]) {
    if hotspots.is_empty() {
        println!("No element spikes found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Selector", "Count"]);

    for (selector, count) in hotspots {
        table.add_row(vec![
            Cell::new(selector),
            Cell::new(format!("{} spikes", count)),
        ]);
    }

    println!("{table}");
}

pub fn print_reviewers_table(reviewers: &[(String, usize)]) {
    if reviewers.is_empty() {
        println!("No reviewers found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Reviewer", "Spikes"]);

    for (name, count) in reviewers {
        table.add_row(vec![
            Cell::new(name),
            Cell::new(format!("{} spikes", count)),
        ]);
    }

    println!("{table}");
}
