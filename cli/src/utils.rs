use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::Client;
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    pub date: String,
    pub description: String,
    pub commit_id: String,
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

pub fn add_commit(path: &str, commit: Commit) -> std::io::Result<()> {
    let commit_string = fs::read_to_string(path)?;
    let mut commits = serde_json::from_str::<Vec<Commit>>(&commit_string).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })?;

    commits.push(commit);

    File::create(path)?.write_all(
        serde_json::to_string(&commits)
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse metadata: {}", e),
                )
            })?
            .as_bytes(),
    )?;

    Ok(())
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

pub fn load_commits() -> std::io::Result<Vec<Commit>> {
    let history_path = ".history".to_owned();
    let commits_string_result = fs::read_to_string(history_path + "/commits.json")?;
    serde_json::from_str::<Vec<Commit>>(&commits_string_result).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
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

pub fn load_ignore_file() -> Vec<String> {
    let mut ignore_file = match File::open(".ignore") {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    let mut ignore_file_contents = String::new();
    ignore_file
        .read_to_string(&mut ignore_file_contents)
        .unwrap();
    let ignore_file_lines: Vec<&str> = ignore_file_contents.split("\n").collect();
    let mut ignore_file_lines_without_comments = Vec::new();

    for line in ignore_file_lines {
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        ignore_file_lines_without_comments.push(line.to_owned());
    }

    ignore_file_lines_without_comments
}

pub fn traverse_directory(path: Option<&Path>) -> Vec<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => Path::new("."),
    };
    let mut result = Vec::new();
    let files_to_ignore = load_ignore_file();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name().unwrap().to_str().unwrap();
                if files_to_ignore.contains(&entry_name.to_owned()) {
                    continue;
                }
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
    let files_to_ignore = load_ignore_file();

    for entry_path in files {
        let entry_name = entry_path.file_name().unwrap().to_str().unwrap();
        if files_to_ignore.contains(&entry_name.to_owned()) {
            continue;
        }
        if entry_path.is_file() {
            fs::remove_file(entry_path)?;
        } else if entry_path.is_dir() {
            fs::remove_dir_all(entry_path)?;
        }
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

pub async fn get_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
) -> Result<GetObjectOutput, io::Error> {
    client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to get object: {}", e),
            )
        })
}

pub async fn create_file_from_s3object(
    client: &Client,
    destination: &str,
    bucket_name: &str,
    key: &str,
) -> io::Result<()> {
    fs::create_dir_all(Path::new(destination).parent().unwrap())?;

    let mut file = File::create(destination)?;

    let mut object = get_object(client, bucket_name, key).await?;

    while let Some(bytes) = object.body.try_next().await? {
        file.write_all(&bytes)?;
    }

    Ok(())
}
