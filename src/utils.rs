use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommitMetadata {
    pub date: String,
    pub description: String,
    pub commit_id: String,
    pub pointer_to_data: i32,
    pub size: i32,
}

pub fn check_if_initialized() -> std::io::Result<()> {
    if !fs::read_dir(".history").is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Repository not initialized",
        ));
    }

    Ok(())
}

pub fn write_to_metadata_file(path: &str, metadata: Vec<CommitMetadata>) -> std::io::Result<()> {
    let mut metadata_file = File::create(path.to_owned() + "/metadata.json")?;
    let metadata_string = serde_json::to_string(&metadata)?;
    metadata_file.write_all(metadata_string.as_bytes())?;
    Ok(())
}

pub fn write_to_data_file(path: &str, data: &str, exists: bool) -> std::io::Result<()> {
    if exists {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.to_owned() + "/data.bin")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    } else {
        let mut file = File::create(path.to_owned() + "/data.bin")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}

pub fn file_metadata(path: &str) -> std::io::Result<Vec<CommitMetadata>> {
    let file_metadata_string_result = fs::read_to_string(path.to_owned() + "/metadata.json")?;
    serde_json::from_str(&file_metadata_string_result).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
}

pub fn generate_commit_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub fn get_current_formatted_date() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn traverse_directory(path: Option<&Path>) -> Vec<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => Path::new("."),
    };
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    result.push(entry_path.clone());
                } else if entry_path.is_dir() {
                    if entry_path.file_name().unwrap().to_str().unwrap() == ".history" {
                        continue;
                    }
                    let subdirectory_files = traverse_directory(Some(&entry_path));
                    result.extend(subdirectory_files);
                }
            }
        }
    }

    result
}

pub fn delete_contents_of_directory(path: &str) -> io::Result<()> {
    let files = traverse_directory(Some(Path::new(path)));

    for file in files {
        fs::remove_file(file)?;
    }

    Ok(())
}

pub fn compare_strings(a: &str, b: &str) -> bool {
    a == b
}

pub fn read_part_of_file(file_path: &str, start: u64, length: usize) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0; length];
    file.read_exact(&mut buffer)?;

    let contents = String::from_utf8(buffer).unwrap();

    Ok(contents)
}

pub fn find_metadata_by_commit_id(
    metadata: &Vec<CommitMetadata>,
    commit_id: &str,
) -> Option<CommitMetadata> {
    for commit_metadata in metadata {
        if commit_metadata.commit_id == commit_id {
            return Some(commit_metadata.clone());
        }
    }

    None
}
