pub mod dates;
pub mod fs_provider;
pub mod s3_provider;

use self::fs_provider::{read_part_of_file, traverse_directory};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::{self};
use std::io::{self, Read, Write};
use std::path::Path;

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

pub fn generate_commit_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
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

pub fn file_metadata(path: &str) -> std::io::Result<Vec<CommitMetadata>> {
    let file_metadata_string_result = fs::read_to_string(path.to_owned() + "/metadata.json")?;
    serde_json::from_str(&file_metadata_string_result).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
}

pub fn load_commits(path: &str) -> std::io::Result<Vec<Commit>> {
    let history_path = ".history".to_owned();
    let commits_string_result = fs::read_to_string(history_path + path)?;
    serde_json::from_str::<Vec<Commit>>(&commits_string_result).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
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

pub fn load_commit(branch_id: &str) -> std::io::Result<()> {
    let history_path = ".history".to_owned();

    if let Ok(entries) = fs::read_dir(history_path.clone()) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    continue;
                }
                let file_name = entry_path.file_name().unwrap().to_str().unwrap().to_owned();
                let file_name_for_history = file_name.replace("_", "/");
                let last_committed_file_path: String = history_path.clone() + &"/" + &file_name;

                let file_metadata = file_metadata(&last_committed_file_path)?;
                let target_commit_metadata_result =
                    find_metadata_by_commit_id(&file_metadata, branch_id);
                let target_commit_metadata = match target_commit_metadata_result {
                    None => file_metadata.last().unwrap().clone(),
                    _ => target_commit_metadata_result.unwrap(),
                };

                let last_committed_file_pointer = target_commit_metadata.pointer_to_data;
                let last_committed_file_size = target_commit_metadata.size;

                let last_committed_file_contents = read_part_of_file(
                    &(last_committed_file_path.clone() + "/data.bin"),
                    last_committed_file_pointer as u64,
                    last_committed_file_size as usize,
                )?;

                fs::create_dir_all(Path::new(&file_name_for_history).parent().unwrap())?;
                let mut file = File::create(file_name_for_history)?;
                file.write_all(last_committed_file_contents.as_bytes())?;
            }
        }
    }

    Ok(())
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

// TODO: Make generic
pub fn find_commit_by_commit_id(metadata: &Vec<Commit>, commit_id: &str) -> Option<Commit> {
    for commit_metadata in metadata {
        if commit_metadata.commit_id == commit_id {
            return Some(commit_metadata.clone());
        }
    }

    None
}

pub fn load_ignores() -> Vec<String> {
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

pub fn write_to_metadata_file(path: &str, metadata: Vec<CommitMetadata>) -> std::io::Result<()> {
    let mut metadata_file = File::create(path.to_owned() + "/metadata.json")?;
    let metadata_string = serde_json::to_string(&metadata)?;
    metadata_file.write_all(metadata_string.as_bytes())?;
    Ok(())
}
