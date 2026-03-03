use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::{Error, Result};
use crate::spike::Spike;

const FEEDBACK_FILE: &str = ".spikes/feedback.jsonl";
const MIN_PREFIX_LENGTH: usize = 4;

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

/// Save all spikes back to the JSONL file.
pub fn save_spikes(spikes: &[Spike]) -> Result<()> {
    let path = Path::new(FEEDBACK_FILE);

    if !path.exists() {
        return Err(Error::NoSpikesDir);
    }

    let mut file = fs::File::create(path)?;
    for spike in spikes {
        writeln!(file, "{}", serde_json::to_string(spike)?)?;
    }

    Ok(())
}

/// Find a spike by its full ID or a unique prefix.
/// 
/// # Arguments
/// * `id_or_prefix` - Full ID or prefix (minimum 4 characters)
/// 
/// # Returns
/// The matching spike, or an error if not found or ambiguous.
pub fn find_spike_by_id(spikes: &[Spike], id_or_prefix: &str) -> Result<Spike> {
    // Validate minimum prefix length
    if id_or_prefix.len() < MIN_PREFIX_LENGTH {
        return Err(Error::SpikeNotFound(format!(
            "ID prefix must be at least {} characters, got '{}'",
            MIN_PREFIX_LENGTH, id_or_prefix
        )));
    }

    // Find all matches
    let matches: Vec<&Spike> = spikes
        .iter()
        .filter(|s| s.id == id_or_prefix || s.id.starts_with(id_or_prefix))
        .collect();

    match matches.len() {
        0 => Err(Error::SpikeNotFound(id_or_prefix.to_string())),
        1 => Ok(matches[0].clone()),
        _ => {
            let matching_ids: Vec<&str> = matches.iter().map(|s| s.id.as_str()).collect();
            Err(Error::SpikeNotFound(format!(
                "Ambiguous ID prefix '{}'. Matches: {}",
                id_or_prefix,
                matching_ids.join(", ")
            )))
        }
    }
}

/// Remove a spike by ID and save the updated list.
/// 
/// # Arguments
/// * `id_or_prefix` - Full ID or prefix (minimum 4 characters)
/// 
/// # Returns
/// The removed spike, or an error if not found.
pub fn remove_spike(id_or_prefix: &str) -> Result<Spike> {
    let mut spikes = load_spikes()?;
    let spike = find_spike_by_id(&spikes, id_or_prefix)?;
    
    spikes.retain(|s| s.id != spike.id);
    save_spikes(&spikes)?;
    
    Ok(spike)
}

/// Update a spike in place and save.
/// 
/// # Arguments
/// * `id_or_prefix` - Full ID or prefix (minimum 4 characters)
/// * `updater` - Function to modify the spike
/// 
/// # Returns
/// The updated spike, or an error if not found.
pub fn update_spike<F>(id_or_prefix: &str, updater: F) -> Result<Spike>
where
    F: FnOnce(&mut Spike),
{
    let mut spikes = load_spikes()?;
    let spike = find_spike_by_id(&spikes, id_or_prefix)?;
    
    // Find and update the spike
    for s in &mut spikes {
        if s.id == spike.id {
            updater(s);
            let updated = s.clone();
            save_spikes(&spikes)?;
            return Ok(updated);
        }
    }
    
    // This shouldn't happen since find_spike_by_id succeeded
    Err(Error::SpikeNotFound(spike.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    use std::sync::Mutex;

    // Use a mutex to prevent parallel test execution for tests that change current directory
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_load_empty_file() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        // Create empty file
        fs::File::create(spikes_dir.join("feedback.jsonl")).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let spikes = load_spikes().unwrap();
        assert!(spikes.is_empty());

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_load_single_spike() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let spike_json = "{\"id\":\"test-1\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"Test\"},\"rating\":\"like\",\"comments\":\"Good\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        writeln!(file, "{}", spike_json).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let spikes = load_spikes().unwrap();
        assert_eq!(spikes.len(), 1);
        assert_eq!(spikes[0].id, "test-1");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_load_multiple_spikes() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let spike1 = "{\"id\":\"spike-1\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"A\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        let spike2 = "{\"id\":\"spike-2\",\"type\":\"element\",\"projectKey\":\"p\",\"page\":\"page.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r2\",\"name\":\"B\"},\"rating\":\"love\",\"comments\":\"B\",\"timestamp\":\"2024-01-01T00:01:00Z\"}";
        let spike3 = "{\"id\":\"spike-3\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"about.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"meh\",\"comments\":\"C\",\"timestamp\":\"2024-01-01T00:02:00Z\"}";
        writeln!(file, "{}", spike1).unwrap();
        writeln!(file, "{}", spike2).unwrap();
        writeln!(file, "{}", spike3).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let spikes = load_spikes().unwrap();
        assert_eq!(spikes.len(), 3);
        assert_eq!(spikes[0].id, "spike-1");
        assert_eq!(spikes[1].id, "spike-2");
        assert_eq!(spikes[2].id, "spike-3");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_load_skips_empty_lines() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let spike1 = "{\"id\":\"spike-1\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"A\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        let spike2 = "{\"id\":\"spike-2\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"B\",\"timestamp\":\"2024-01-01T00:01:00Z\"}";
        writeln!(file, "{}", spike1).unwrap();
        writeln!(file, "").unwrap();  // Empty line
        writeln!(file, "   ").unwrap();  // Whitespace only
        writeln!(file, "{}", spike2).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let spikes = load_spikes().unwrap();
        assert_eq!(spikes.len(), 2);

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_load_missing_directory() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = load_spikes();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoSpikesDir));

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_load_malformed_json() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let valid_spike = "{\"id\":\"valid\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"A\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        writeln!(file, "{}", valid_spike).unwrap();
        writeln!(file, "{{invalid json").unwrap();  // Malformed

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = load_spikes();
        assert!(result.is_err());

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_save_spikes() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        // Create initial file
        let feedback_path = spikes_dir.join("feedback.jsonl");
        fs::File::create(&feedback_path).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let spikes = vec![
            Spike {
                id: "save-test-1".to_string(),
                spike_type: crate::spike::SpikeType::Page,
                project_key: "p".to_string(),
                page: "index.html".to_string(),
                url: "http://test".to_string(),
                reviewer: crate::spike::Reviewer { id: "r1".to_string(), name: "Test".to_string() },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: Some(crate::spike::Rating::Like),
                comments: "Good".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
        ];

        save_spikes(&spikes).unwrap();

        // Verify the file was written
        let loaded = load_spikes().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "save-test-1");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_find_spike_by_full_id() {
        let spikes = vec![
            Spike {
                id: "abcdef123456".to_string(),
                spike_type: crate::spike::SpikeType::Page,
                project_key: "p".to_string(),
                page: "index.html".to_string(),
                url: "http://test".to_string(),
                reviewer: crate::spike::Reviewer { id: "r1".to_string(), name: "Test".to_string() },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: None,
                comments: "".to_string(),
                timestamp: "".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
        ];

        let result = find_spike_by_id(&spikes, "abcdef123456").unwrap();
        assert_eq!(result.id, "abcdef123456");
    }

    #[test]
    fn test_find_spike_by_prefix() {
        let spikes = vec![
            Spike {
                id: "abcdefgh1234".to_string(),
                spike_type: crate::spike::SpikeType::Page,
                project_key: "p".to_string(),
                page: "index.html".to_string(),
                url: "http://test".to_string(),
                reviewer: crate::spike::Reviewer { id: "r1".to_string(), name: "Test".to_string() },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: None,
                comments: "".to_string(),
                timestamp: "".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
        ];

        let result = find_spike_by_id(&spikes, "abcd").unwrap();
        assert_eq!(result.id, "abcdefgh1234");
    }

    #[test]
    fn test_find_spike_prefix_too_short() {
        let spikes = vec![];

        let result = find_spike_by_id(&spikes, "abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 4 characters"));
    }

    #[test]
    fn test_find_spike_ambiguous_prefix() {
        let spikes = vec![
            Spike {
                id: "abcdef123456".to_string(),
                spike_type: crate::spike::SpikeType::Page,
                project_key: "p".to_string(),
                page: "index.html".to_string(),
                url: "http://test".to_string(),
                reviewer: crate::spike::Reviewer { id: "r1".to_string(), name: "Test".to_string() },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: None,
                comments: "".to_string(),
                timestamp: "".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
            Spike {
                id: "abcdef789012".to_string(),
                spike_type: crate::spike::SpikeType::Page,
                project_key: "p".to_string(),
                page: "page.html".to_string(),
                url: "http://test".to_string(),
                reviewer: crate::spike::Reviewer { id: "r1".to_string(), name: "Test".to_string() },
                selector: None,
                element_text: None,
                bounding_box: None,
                rating: None,
                comments: "".to_string(),
                timestamp: "".to_string(),
                viewport: None,
                resolved: None,
                resolved_at: None,
            },
        ];

        let result = find_spike_by_id(&spikes, "abcd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ambiguous"));
    }

    #[test]
    fn test_find_spike_not_found() {
        let spikes = vec![];

        let result = find_spike_by_id(&spikes, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::SpikeNotFound(_)));
    }

    #[test]
    fn test_remove_spike() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let spike1 = "{\"id\":\"delete-test-1\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"A\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        let spike2 = "{\"id\":\"delete-test-2\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"page.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"B\",\"timestamp\":\"2024-01-01T00:01:00Z\"}";
        writeln!(file, "{}", spike1).unwrap();
        writeln!(file, "{}", spike2).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = remove_spike("delete-test-1").unwrap();
        assert_eq!(removed.id, "delete-test-1");

        let remaining = load_spikes().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "delete-test-2");

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[test]
    fn test_update_spike() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let spikes_dir = temp_dir.path().join(".spikes");
        fs::create_dir_all(&spikes_dir).unwrap();

        let feedback_path = spikes_dir.join("feedback.jsonl");
        let mut file = fs::File::create(&feedback_path).unwrap();
        let spike = "{\"id\":\"update-test-1\",\"type\":\"page\",\"projectKey\":\"p\",\"page\":\"index.html\",\"url\":\"http://test\",\"reviewer\":{\"id\":\"r1\",\"name\":\"A\"},\"rating\":\"like\",\"comments\":\"A\",\"timestamp\":\"2024-01-01T00:00:00Z\"}";
        writeln!(file, "{}", spike).unwrap();

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let updated = update_spike("update-test-1", |s| {
            s.resolved = Some(true);
            s.resolved_at = Some("2024-01-02T00:00:00Z".to_string());
        }).unwrap();

        assert_eq!(updated.resolved, Some(true));
        assert_eq!(updated.resolved_at, Some("2024-01-02T00:00:00Z".to_string()));

        // Verify persistence
        let loaded = load_spikes().unwrap();
        assert_eq!(loaded[0].resolved, Some(true));

        std::env::set_current_dir(original_cwd).unwrap();
    }
}
