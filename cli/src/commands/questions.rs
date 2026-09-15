//! `spikes questions add|list|answers|close` — ask reviewers questions and
//! read their answers.

use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

use crate::api::{data_array, encode, require_project_key, with_query, ApiClient};
use crate::error::{Error, Result};
use crate::output::print_json;

fn questions_path(project: &str) -> String {
    format!("/me/projects/{}/questions", encode(project))
}

pub fn add(title: &str, body: Option<String>, json: bool) -> Result<()> {
    if title.trim().is_empty() {
        return Err(Error::RequestFailed(
            "Question title cannot be empty".to_string(),
        ));
    }
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;

    let mut payload = serde_json::json!({ "title": title });
    if let Some(ref b) = body {
        payload["body"] = serde_json::Value::String(b.clone());
    }

    let response = client.post(&questions_path(&project), &payload)?;

    if json {
        print_json(&serde_json::json!({ "success": true, "question": response }));
    } else {
        let id = response.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        println!();
        println!("  / Question added to '{}'", project);
        println!();
        println!("  ID:     {}", id);
        println!("  Title:  {}", title);
        if let Some(ref b) = body {
            println!("  Body:   {}", b);
        }
        println!();
    }
    Ok(())
}

pub fn list(closed: bool, json: bool) -> Result<()> {
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;
    let status = if closed { "closed" } else { "open" };
    let path = with_query(&questions_path(&project), &[("status", Some(status))]);
    let response = client.get(&path)?;
    let questions = data_array(&response);

    if json {
        print_json(&questions);
        return Ok(());
    }

    if questions.is_empty() {
        println!();
        println!("  No {} questions for '{}'.", status, project);
        println!();
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Title", "Answers", "Last answer"]);

    for q in &questions {
        let id = q.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        let title = q.get("title").and_then(|v| v.as_str()).unwrap_or("-");
        let answers = q
            .get("answerCount")
            .or_else(|| q.get("answer_count"))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0".to_string());
        let last = q
            .get("lastAnswer")
            .or_else(|| q.get("last_answer"))
            .map(|a| {
                let who = a
                    .get("reviewerName")
                    .or_else(|| a.get("reviewer_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let body = a.get("body").and_then(|v| v.as_str()).unwrap_or("");
                format!("{}: {}", who, truncate(body, 60))
            })
            .unwrap_or_else(|| "-".to_string());
        table.add_row(vec![
            Cell::new(id),
            Cell::new(title),
            Cell::new(answers),
            Cell::new(last),
        ]);
    }

    println!();
    println!("{}", table);
    println!();
    Ok(())
}

pub fn answers(question_id: &str, json: bool) -> Result<()> {
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;
    let response = client.get(&format!(
        "{}/{}/answers",
        questions_path(&project),
        encode(question_id)
    ))?;
    let answers = data_array(&response);

    if json {
        print_json(&answers);
        return Ok(());
    }

    if answers.is_empty() {
        println!();
        println!("  No answers yet for question {}.", question_id);
        println!();
        return Ok(());
    }

    println!();
    for a in &answers {
        let who = a
            .get("reviewer")
            .and_then(|r| r.get("name"))
            .or_else(|| a.get("reviewerName"))
            .and_then(|v| v.as_str())
            .unwrap_or("Anonymous");
        let when = a
            .get("createdAt")
            .or_else(|| a.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let body = a.get("body").and_then(|v| v.as_str()).unwrap_or("");
        println!("  {} ({})", who, when);
        for line in body.lines() {
            println!("    {}", line);
        }
        println!();
    }
    Ok(())
}

pub fn close(question_id: &str, json: bool) -> Result<()> {
    let project = require_project_key()?;
    let client = ApiClient::from_env()?;
    let body = serde_json::json!({ "status": "closed" });
    let response = client.patch(
        &format!("{}/{}", questions_path(&project), encode(question_id)),
        &body,
    )?;

    if json {
        print_json(&serde_json::json!({ "success": true, "question": response }));
    } else {
        println!();
        println!("  / Question {} closed", question_id);
        println!();
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghijk", 5), "abcd…");
    }

    #[test]
    fn test_questions_path_encodes() {
        assert_eq!(
            questions_path("my proj"),
            "/me/projects/my%20proj/questions"
        );
    }
}
