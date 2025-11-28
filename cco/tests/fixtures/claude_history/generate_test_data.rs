//! Test data generator for performance benchmarks
//!
//! This helper generates large JSONL files for performance testing.
//!
//! Usage:
//!   cargo run --bin generate_test_data -- --files 100 --messages-per-file 1000 --output /tmp/test_data

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_test_file(path: &PathBuf, num_messages: usize, model: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    for i in 0..num_messages {
        let line = format!(
            r#"{{"type":"assistant","message":{{"model":"{}","usage":{{"input_tokens":{},"output_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            model,
            (i + 1) * 100,
            (i + 1) * 50
        );
        file.write_all(line.as_bytes())?;
    }

    file.flush()?;
    Ok(())
}

pub fn generate_multi_file_dataset(
    output_dir: &PathBuf,
    num_files: usize,
    messages_per_file: usize,
) -> std::io::Result<()> {
    std::fs::create_dir_all(output_dir)?;

    let models = vec![
        "claude-opus-4-20250514",
        "claude-sonnet-4-5-20250929",
        "claude-haiku-4-5-20251001",
        "claude-3-5-sonnet-20241022",
    ];

    for i in 0..num_files {
        let filename = output_dir.join(format!("conversation_{:04}.jsonl", i));
        let model = models[i % models.len()];
        generate_test_file(&filename, messages_per_file, model)?;

        if i % 10 == 0 {
            println!("Generated {} files...", i);
        }
    }

    println!("Generated {} files with {} messages each", num_files, messages_per_file);
    Ok(())
}

pub fn generate_large_single_file(path: &PathBuf, num_messages: usize) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    let models = vec![
        "claude-opus-4-20250514",
        "claude-sonnet-4-5-20250929",
        "claude-haiku-4-5-20251001",
    ];

    for i in 0..num_messages {
        let model = models[i % models.len()];
        let line = format!(
            r#"{{"type":"assistant","message":{{"model":"{}","usage":{{"input_tokens":{},"output_tokens":{},"cache_creation_input_tokens":{},"cache_read_input_tokens":{}}}}},"timestamp":"2025-11-26T10:00:00Z"}}
"#,
            model,
            (i + 1) * 100,
            (i + 1) * 50,
            if i % 5 == 0 { (i + 1) * 200 } else { 0 },
            if i % 3 == 0 { (i + 1) * 1000 } else { 0 }
        );
        file.write_all(line.as_bytes())?;

        if i % 1000 == 0 {
            println!("Written {} messages...", i);
        }
    }

    file.flush()?;
    println!("Generated file with {} messages", num_messages);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_test_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jsonl");

        generate_test_file(&test_file, 10, "claude-sonnet-4-5-20250929").unwrap();

        let content = std::fs::read_to_string(&test_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn test_generate_multi_file_dataset() {
        let temp_dir = TempDir::new().unwrap();

        generate_multi_file_dataset(&temp_dir.path().to_path_buf(), 5, 10).unwrap();

        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .collect();
        assert_eq!(entries.len(), 5);
    }
}
