use std::collections::HashMap;
use std::io::{self, Write};

use crate::error::Result;
use crate::spike::{Rating, SpikeType};
use crate::storage::load_spikes;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Jsonl,
    CursorContext,
    ClaudeContext,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            "jsonl" => Ok(ExportFormat::Jsonl),
            "cursor-context" => Ok(ExportFormat::CursorContext),
            "claude-context" => Ok(ExportFormat::ClaudeContext),
            _ => Err(format!(
                "Invalid format: {}. Use json, csv, jsonl, cursor-context, or claude-context",
                s
            )),
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
                    &spike.viewport.as_ref().map(|v| v.width.to_string()).unwrap_or_default(),
                    &spike.viewport.as_ref().map(|v| v.height.to_string()).unwrap_or_default(),
                ])?;
            }
            wtr.flush()?;
        }
        ExportFormat::CursorContext => {
            let markdown = generate_cursor_context(&spikes);
            write!(handle, "{}", markdown)?;
        }
        ExportFormat::ClaudeContext => {
            let markdown = generate_claude_context(&spikes);
            write!(handle, "{}", markdown)?;
        }
    }

    Ok(())
}

// ============================================================================
// Cursor Context Format
// ============================================================================

/// Generate Cursor-compatible context markdown.
///
/// Sections: blocking issues, hotspots, element-specific notes.
/// Punk/zine energy in headers and taglines.
fn generate_cursor_context(spikes: &[crate::spike::Spike]) -> String {
    let mut output = String::new();

    // Metadata header
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let project = spikes
        .first()
        .map(|s| s.project_key.as_str())
        .unwrap_or("unknown");

    output.push_str("# 🎯 FEEDBACK INTEL\n\n");
    output.push_str("_Your roadmap to glory or ruin._\n\n");
    output.push_str(&format!("**Project:** {}\n", project));
    output.push_str(&format!("**Total Spikes:** {}\n", spikes.len()));
    output.push_str(&format!("**Generated:** {}\n\n", timestamp));
    output.push_str("---\n\n");

    // Blocking issues section
    output.push_str("## 🚫 BLOCKING ISSUES\n\n");
    output.push_str("_The vibes are off. Fix these before shipping._\n\n");

    let blocking: Vec<&crate::spike::Spike> = spikes
        .iter()
        .filter(|s| is_blocking(s))
        .collect();

    if blocking.is_empty() {
        output.push_str("✨ **Clean slate!** No blocking issues found.\n\n");
    } else {
        for spike in &blocking {
            output.push_str(&format!("### [{}] {} on `{}`\n", 
                &spike.id.chars().take(8).collect::<String>(),
                spike.type_str(),
                spike.page
            ));
            output.push_str(&format!("- **Rating:** {}\n", spike.rating_str()));
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    output.push_str(&format!("- **Selector:** `{}`\n", selector));
                }
            }
            if !spike.comments.is_empty() {
                output.push_str(&format!("- **Comment:** \"{}\"\n", spike.comments));
            }
            output.push_str(&format!("- **Reviewer:** {}\n", spike.reviewer.name));
            output.push('\n');
        }
    }

    output.push_str("---\n\n");

    // Hotspots section
    output.push_str("## 🔥 FEEDBACK HOTSPOTS\n\n");
    output.push_str("_Where the action is. Elements with the most heat._\n\n");

    let hotspots = compute_hotspots(spikes);
    if hotspots.is_empty() {
        output.push_str("📊 **No element feedback.** Nothing's hot yet.\n\n");
    } else {
        for (i, (selector, count)) in hotspots.iter().enumerate() {
            output.push_str(&format!(
                "{}. `{}` — **{} spike{}**\n",
                i + 1,
                selector,
                count,
                if *count == 1 { "" } else { "s" }
            ));
        }
        output.push('\n');
    }

    output.push_str("---\n\n");

    // Element-specific notes section
    output.push_str("## 📝 ELEMENT-SPECIFIC NOTES\n\n");
    output.push_str("_Deep cuts on specific elements. Grouped by selector._\n\n");

    let element_spikes: Vec<&crate::spike::Spike> = spikes
        .iter()
        .filter(|s| s.spike_type == SpikeType::Element)
        .collect();

    if element_spikes.is_empty() {
        output.push_str("🔍 **No element feedback recorded.**\n\n");
    } else {
        // Group by selector
        let mut by_selector: HashMap<String, Vec<&crate::spike::Spike>> = HashMap::new();
        for spike in &element_spikes {
            let selector = spike.selector.clone().unwrap_or_default();
            by_selector.entry(selector).or_default().push(*spike);
        }

        // Sort selectors alphabetically for consistent output
        let mut selectors: Vec<String> = by_selector.keys().cloned().collect();
        selectors.sort();

        for selector in &selectors {
            let spikes_for_selector = &by_selector[selector];
            output.push_str(&format!("### `{}`\n\n", selector));

            for spike in spikes_for_selector {
                let status = if spike.is_resolved() { "✅" } else { "⏳" };
                output.push_str(&format!(
                    "- {} **{}** — \"{}\" _({})_\n",
                    status,
                    spike.rating_str(),
                    spike.comments,
                    spike.reviewer.name
                ));
            }
            output.push('\n');
        }
    }

    output.push_str("---\n\n");
    output.push_str("_Generated by [spikes](https://spikes.sh) — feedback that talks back._\n");

    output
}

// ============================================================================
// Claude Context Format
// ============================================================================

/// Generate Claude-compatible context markdown.
///
/// Sections: critical issues, feedback hotspots, element feedback.
/// Distinct punk/zine tone from cursor-context.
fn generate_claude_context(spikes: &[crate::spike::Spike]) -> String {
    let mut output = String::new();

    // Metadata header
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let project = spikes
        .first()
        .map(|s| s.project_key.as_str())
        .unwrap_or("unknown");

    output.push_str("# ⚡ SPIKES FEEDBACK REPORT\n\n");
    output.push_str("_The raw truth, served fresh._\n\n");
    output.push_str(&format!("**Project:** {}\n", project));
    output.push_str(&format!("**Total Feedback Items:** {}\n", spikes.len()));
    output.push_str(&format!("**Generated:** {}\n\n", timestamp));
    output.push_str("---\n\n");

    // Critical issues section
    output.push_str("## ⚠️ CRITICAL ISSUES\n\n");
    output.push_str("_Unresolved problems demanding attention. The must-fix list._\n\n");

    let blocking: Vec<&crate::spike::Spike> = spikes
        .iter()
        .filter(|s| is_blocking(s))
        .collect();

    if blocking.is_empty() {
        output.push_str("**✅ All clear.** No critical issues blocking progress.\n\n");
    } else {
        output.push_str(&format!("**{} critical issue{} found:**\n\n", 
            blocking.len(),
            if blocking.len() == 1 { "" } else { "s" }
        ));

        for spike in &blocking {
            output.push_str(&format!("### ID: `{}`\n\n", 
                &spike.id.chars().take(8).collect::<String>()
            ));
            output.push_str(&format!("- **Type:** {} on page `{}`\n", 
                spike.type_str(),
                spike.page
            ));
            output.push_str(&format!("- **Rating:** {} (negative)\n", spike.rating_str()));
            if spike.spike_type == SpikeType::Element {
                if let Some(selector) = &spike.selector {
                    output.push_str(&format!("- **Target:** `{}`\n", selector));
                }
            }
            if !spike.comments.is_empty() {
                output.push_str(&format!("- **Feedback:** \"{}\"\n", spike.comments));
            }
            output.push_str(&format!("- **From:** {}\n", spike.reviewer.name));
            output.push('\n');
        }
    }

    output.push_str("---\n\n");

    // Hotspots section
    output.push_str("## 📊 FEEDBACK HOTSPOTS\n\n");
    output.push_str("_Where reviewers clustered. The conversation starters._\n\n");

    let hotspots = compute_hotspots(spikes);
    if hotspots.is_empty() {
        output.push_str("**No element hotspots.** Reviewers haven't targeted specific elements yet.\n\n");
    } else {
        output.push_str("**Top feedback targets:**\n\n");
        for (i, (selector, count)) in hotspots.iter().enumerate() {
            output.push_str(&format!(
                "{}. `{}` — {} feedback item{}\n",
                i + 1,
                selector,
                count,
                if *count == 1 { "" } else { "s" }
            ));
        }
        output.push('\n');
    }

    output.push_str("---\n\n");

    // Element feedback section
    output.push_str("## 🔍 ELEMENT FEEDBACK\n\n");
    output.push_str("_Granular feedback on specific components. Organized by selector._\n\n");

    let element_spikes: Vec<&crate::spike::Spike> = spikes
        .iter()
        .filter(|s| s.spike_type == SpikeType::Element)
        .collect();

    if element_spikes.is_empty() {
        output.push_str("**No element-level feedback recorded.**\n\n");
    } else {
        // Group by selector
        let mut by_selector: HashMap<String, Vec<&crate::spike::Spike>> = HashMap::new();
        for spike in &element_spikes {
            let selector = spike.selector.clone().unwrap_or_default();
            by_selector.entry(selector).or_default().push(*spike);
        }

        // Sort selectors alphabetically
        let mut selectors: Vec<String> = by_selector.keys().cloned().collect();
        selectors.sort();

        for selector in &selectors {
            let spikes_for_selector = &by_selector[selector];
            output.push_str(&format!("### Selector: `{}`\n\n", selector));

            for spike in spikes_for_selector {
                let resolved_marker = if spike.is_resolved() { "[RESOLVED] " } else { "" };
                output.push_str(&format!(
                    "- {}**{}** from {}: \"{}\"\n",
                    resolved_marker,
                    spike.rating_str(),
                    spike.reviewer.name,
                    spike.comments
                ));
            }
            output.push('\n');
        }
    }

    output.push_str("---\n\n");
    output.push_str("_Spikes — structured feedback for the modern builder._\n");

    output
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if a spike is blocking (unresolved with meh/no rating)
fn is_blocking(spike: &crate::spike::Spike) -> bool {
    !spike.is_resolved()
        && matches!(
            spike.rating,
            Some(Rating::Meh) | Some(Rating::No)
        )
}

/// Compute hotspots: element-type spikes counted by selector, sorted descending
fn compute_hotspots(spikes: &[crate::spike::Spike]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for spike in spikes {
        if spike.spike_type == SpikeType::Element {
            if let Some(selector) = &spike.selector {
                *counts.entry(selector.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut hotspots: Vec<(String, usize)> = counts.into_iter().collect();
    hotspots.sort_by(|a, b| b.1.cmp(&a.1));
    hotspots
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spike::{Rating, Reviewer, Spike, SpikeType, Viewport};

    // Helper to create a test spike
    fn create_spike(
        id: &str,
        spike_type: SpikeType,
        page: &str,
        rating: Option<Rating>,
        selector: Option<&str>,
        resolved: bool,
        comments: &str,
    ) -> Spike {
        Spike {
            id: id.to_string(),
            spike_type,
            project_key: "test-project".to_string(),
            page: page.to_string(),
            url: format!("http://test/{}", page),
            reviewer: Reviewer {
                id: "r1".to_string(),
                name: "TestReviewer".to_string(),
            },
            selector: selector.map(|s| s.to_string()),
            element_text: None,
            bounding_box: None,
            rating,
            comments: comments.to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            viewport: Some(Viewport {
                width: 1920,
                height: 1080,
            }),
            resolved: if resolved { Some(true) } else { None },
            resolved_at: if resolved {
                Some("2024-01-16T10:00:00Z".to_string())
            } else {
                None
            },
        }
    }

    // ========================================
    // ExportFormat parsing tests
    // ========================================

    #[test]
    fn test_parse_json_format() {
        assert_eq!("json".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
        assert_eq!("JSON".parse::<ExportFormat>().unwrap(), ExportFormat::Json);
    }

    #[test]
    fn test_parse_csv_format() {
        assert_eq!("csv".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
    }

    #[test]
    fn test_parse_jsonl_format() {
        assert_eq!("jsonl".parse::<ExportFormat>().unwrap(), ExportFormat::Jsonl);
    }

    #[test]
    fn test_parse_cursor_context_format() {
        assert_eq!(
            "cursor-context".parse::<ExportFormat>().unwrap(),
            ExportFormat::CursorContext
        );
        assert_eq!(
            "CURSOR-CONTEXT".parse::<ExportFormat>().unwrap(),
            ExportFormat::CursorContext
        );
    }

    #[test]
    fn test_parse_claude_context_format() {
        assert_eq!(
            "claude-context".parse::<ExportFormat>().unwrap(),
            ExportFormat::ClaudeContext
        );
        assert_eq!(
            "CLAUDE-CONTEXT".parse::<ExportFormat>().unwrap(),
            ExportFormat::ClaudeContext
        );
    }

    #[test]
    fn test_invalid_format_lists_all_five() {
        let result = "invalid".parse::<ExportFormat>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("json"), "Error should list json format");
        assert!(err.contains("csv"), "Error should list csv format");
        assert!(err.contains("jsonl"), "Error should list jsonl format");
        assert!(err.contains("cursor-context"), "Error should list cursor-context format");
        assert!(err.contains("claude-context"), "Error should list claude-context format");
    }

    // ========================================
    // is_blocking tests
    // ========================================

    #[test]
    fn test_is_blocking_meh_unresolved() {
        let spike = create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Meh), None, false, "Not great");
        assert!(is_blocking(&spike), "Unresolved meh should be blocking");
    }

    #[test]
    fn test_is_blocking_no_unresolved() {
        let spike = create_spike("s2", SpikeType::Page, "index.html", Some(Rating::No), None, false, "Bad");
        assert!(is_blocking(&spike), "Unresolved no should be blocking");
    }

    #[test]
    fn test_is_not_blocking_love() {
        let spike = create_spike("s3", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Great");
        assert!(!is_blocking(&spike), "Love should not be blocking");
    }

    #[test]
    fn test_is_not_blocking_like() {
        let spike = create_spike("s4", SpikeType::Page, "index.html", Some(Rating::Like), None, false, "Good");
        assert!(!is_blocking(&spike), "Like should not be blocking");
    }

    #[test]
    fn test_is_not_blocking_resolved_meh() {
        let spike = create_spike("s5", SpikeType::Page, "index.html", Some(Rating::Meh), None, true, "Fixed");
        assert!(!is_blocking(&spike), "Resolved meh should not be blocking");
    }

    #[test]
    fn test_is_not_blocking_resolved_no() {
        let spike = create_spike("s6", SpikeType::Page, "index.html", Some(Rating::No), None, true, "Fixed");
        assert!(!is_blocking(&spike), "Resolved no should not be blocking");
    }

    #[test]
    fn test_is_not_blocking_no_rating() {
        let spike = create_spike("s7", SpikeType::Page, "index.html", None, None, false, "Comment only");
        assert!(!is_blocking(&spike), "No rating should not be blocking");
    }

    // ========================================
    // compute_hotspots tests
    // ========================================

    #[test]
    fn test_hotspots_empty() {
        let spikes = vec![];
        let hotspots = compute_hotspots(&spikes);
        assert!(hotspots.is_empty());
    }

    #[test]
    fn test_hotspots_page_only() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Good"),
        ];
        let hotspots = compute_hotspots(&spikes);
        assert!(hotspots.is_empty(), "Page spikes should not create hotspots");
    }

    #[test]
    fn test_hotspots_single_element() {
        let spikes = vec![
            create_spike("s1", SpikeType::Element, "index.html", Some(Rating::Love), Some(".hero"), false, "Nice"),
        ];
        let hotspots = compute_hotspots(&spikes);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0], (".hero".to_string(), 1));
    }

    #[test]
    fn test_hotspots_sorted_descending() {
        let spikes = vec![
            create_spike("s1", SpikeType::Element, "index.html", Some(Rating::Love), Some(".hero"), false, "1"),
            create_spike("s2", SpikeType::Element, "index.html", Some(Rating::Like), Some(".hero"), false, "2"),
            create_spike("s3", SpikeType::Element, "index.html", Some(Rating::Meh), Some(".hero"), false, "3"),
            create_spike("s4", SpikeType::Element, "index.html", Some(Rating::No), Some(".footer"), false, "4"),
        ];
        let hotspots = compute_hotspots(&spikes);
        assert_eq!(hotspots.len(), 2);
        assert_eq!(hotspots[0], (".hero".to_string(), 3), "Most feedback should be first");
        assert_eq!(hotspots[1], (".footer".to_string(), 1));
    }

    // ========================================
    // cursor-context format tests
    // ========================================

    #[test]
    fn test_cursor_context_empty_state() {
        let spikes = vec![];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("# 🎯 FEEDBACK INTEL"));
        assert!(markdown.contains("No blocking issues"));
        assert!(markdown.contains("No element feedback"));
        assert!(markdown.contains("Total Spikes:** 0"));
    }

    #[test]
    fn test_cursor_context_positive_only() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Amazing"),
            create_spike("s2", SpikeType::Page, "about.html", Some(Rating::Like), None, false, "Good"),
        ];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("Clean slate!"));
        assert!(markdown.contains("No blocking issues"));
    }

    #[test]
    fn test_cursor_context_mixed_ratings() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Great"),
            create_spike("s2", SpikeType::Page, "about.html", Some(Rating::Meh), None, false, "Needs work"),
            create_spike("s3", SpikeType::Element, "index.html", Some(Rating::No), Some(".button"), false, "Broken"),
        ];
        let markdown = generate_cursor_context(&spikes);

        // Blocking section should have meh and no ratings
        assert!(markdown.contains("BLOCKING ISSUES"));
        assert!(markdown.contains("about.html"));
        assert!(markdown.contains(".button"));
        assert!(markdown.contains("meh"));
        assert!(markdown.contains("no"));

        // Love should NOT appear in blocking
        assert!(!markdown.contains("Great"));
    }

    #[test]
    fn test_cursor_context_resolved_negative_not_blocking() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::No), None, true, "Fixed now"),
            create_spike("s2", SpikeType::Page, "about.html", Some(Rating::Meh), None, true, "Resolved"),
        ];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("Clean slate!"));
        assert!(markdown.contains("No blocking issues"));
    }

    #[test]
    fn test_cursor_context_element_grouping() {
        let spikes = vec![
            create_spike("s1", SpikeType::Element, "index.html", Some(Rating::Love), Some(".hero"), false, "Nice hero"),
            create_spike("s2", SpikeType::Element, "index.html", Some(Rating::No), Some(".hero"), false, "Hero broken"),
            create_spike("s3", SpikeType::Element, "about.html", Some(Rating::Like), Some(".footer"), false, "Nice footer"),
        ];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("ELEMENT-SPECIFIC NOTES"));
        assert!(markdown.contains("### `.hero`"));
        assert!(markdown.contains("### `.footer`"));
        // Check that both spikes for .hero appear
        assert!(markdown.contains("Nice hero"));
        assert!(markdown.contains("Hero broken"));
    }

    #[test]
    fn test_cursor_context_hotspots() {
        let spikes = vec![
            create_spike("s1", SpikeType::Element, "index.html", Some(Rating::Love), Some(".hero"), false, "1"),
            create_spike("s2", SpikeType::Element, "index.html", Some(Rating::Like), Some(".hero"), false, "2"),
            create_spike("s3", SpikeType::Element, "index.html", Some(Rating::No), Some(".footer"), false, "3"),
        ];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("FEEDBACK HOTSPOTS"));
        assert!(markdown.contains("`.hero` — **2 spikes**"));
        assert!(markdown.contains("`.footer` — **1 spike**"));
    }

    #[test]
    fn test_cursor_context_punk_zine_tone() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::No), None, false, "Bad"),
        ];
        let markdown = generate_cursor_context(&spikes);

        assert!(markdown.contains("vibes are off"));
        assert!(markdown.contains("Where the action is"));
        assert!(markdown.contains("Deep cuts"));
    }

    // ========================================
    // claude-context format tests
    // ========================================

    #[test]
    fn test_claude_context_empty_state() {
        let spikes = vec![];
        let markdown = generate_claude_context(&spikes);

        assert!(markdown.contains("# ⚡ SPIKES FEEDBACK REPORT"));
        assert!(markdown.contains("No critical issues"));
        assert!(markdown.contains("No element hotspots"));
        assert!(markdown.contains("Total Feedback Items:** 0"));
    }

    #[test]
    fn test_claude_context_positive_only() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Amazing"),
            create_spike("s2", SpikeType::Page, "about.html", Some(Rating::Like), None, false, "Good"),
        ];
        let markdown = generate_claude_context(&spikes);

        assert!(markdown.contains("All clear"));
        assert!(markdown.contains("No critical issues"));
    }

    #[test]
    fn test_claude_context_mixed_ratings() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Great"),
            create_spike("s2", SpikeType::Page, "about.html", Some(Rating::Meh), None, false, "Needs work"),
            create_spike("s3", SpikeType::Element, "index.html", Some(Rating::No), Some(".button"), false, "Broken"),
        ];
        let markdown = generate_claude_context(&spikes);

        assert!(markdown.contains("CRITICAL ISSUES"));
        assert!(markdown.contains("about.html"));
        assert!(markdown.contains(".button"));
        assert!(markdown.contains("negative"));
    }

    #[test]
    fn test_claude_context_distinct_from_cursor() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::No), None, false, "Bad"),
        ];
        let cursor_md = generate_cursor_context(&spikes);
        let claude_md = generate_claude_context(&spikes);

        // Different main headers
        assert!(cursor_md.contains("# 🎯 FEEDBACK INTEL"));
        assert!(claude_md.contains("# ⚡ SPIKES FEEDBACK REPORT"));

        // Different section headers
        assert!(cursor_md.contains("BLOCKING ISSUES"));
        assert!(claude_md.contains("CRITICAL ISSUES"));
    }

    #[test]
    fn test_claude_context_element_feedback_resolved_marker() {
        let spikes = vec![
            create_spike("s1", SpikeType::Element, "index.html", Some(Rating::Love), Some(".hero"), true, "Fixed"),
            create_spike("s2", SpikeType::Element, "index.html", Some(Rating::No), Some(".hero"), false, "Broken"),
        ];
        let markdown = generate_claude_context(&spikes);

        assert!(markdown.contains("[RESOLVED]"));
        assert!(markdown.contains("Fixed"));
        assert!(markdown.contains("Broken"));
    }

    #[test]
    fn test_claude_context_punk_zine_tone() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::No), None, false, "Bad"),
        ];
        let markdown = generate_claude_context(&spikes);

        assert!(markdown.contains("raw truth"));
        assert!(markdown.contains("demanding attention"));
        assert!(markdown.contains("Where reviewers clustered"));
    }

    // ========================================
    // Metadata tests
    // ========================================

    #[test]
    fn test_context_export_includes_metadata() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Good"),
        ];

        let cursor_md = generate_cursor_context(&spikes);
        let claude_md = generate_claude_context(&spikes);

        // Both should have project, count, timestamp
        assert!(cursor_md.contains("**Project:**"));
        assert!(cursor_md.contains("**Total Spikes:**"));
        assert!(cursor_md.contains("**Generated:**"));

        assert!(claude_md.contains("**Project:**"));
        assert!(claude_md.contains("**Total Feedback Items:**"));
        assert!(claude_md.contains("**Generated:**"));
    }

    #[test]
    fn test_context_export_project_from_spikes() {
        let spikes = vec![
            create_spike("s1", SpikeType::Page, "index.html", Some(Rating::Love), None, false, "Good"),
        ];

        let cursor_md = generate_cursor_context(&spikes);
        assert!(cursor_md.contains("test-project"));
    }
}
