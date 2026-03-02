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
}
