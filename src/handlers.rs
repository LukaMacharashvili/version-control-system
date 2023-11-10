use crate::files::*;
use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{self};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CommitMetadata {
    date: String,
    description: String,
    commit_id: String,
    pointer_to_data: i32,
    size: i32,
}

pub fn init() -> std::io::Result<()> {
    let history_path = ".history".to_owned();
    fs::create_dir(history_path)?;

    Ok(())
}

pub fn commit(description: &str) -> std::io::Result<()> {
    if !fs::read_dir(".history").is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Repository not initialized",
        ));
    }

    let history_path = ".history".to_owned();
    let commit_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let file_paths = traverse_directory(None);

    for file_path in file_paths {
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let last_committed_file_path = history_path.clone() + &"/" + file_name;
        let last_committed_file_data_path = last_committed_file_path.clone() + "/data";
        let last_committed_file_metadata_path = last_committed_file_path.clone() + "/metadata.json";
        let file_contents = fs::read_to_string(file_path.clone())?;
        let now: DateTime<Utc> = Utc::now();
        let formatted_date = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let file_metadata_string_result =
            fs::read_to_string(last_committed_file_metadata_path.clone());
        let file_metadata_string = match file_metadata_string_result {
            Ok(string) => string,
            Err(_) => {
                fs::create_dir(&last_committed_file_path)?;

                let mut metadata_file = File::create(last_committed_file_metadata_path.clone())?;

                let metadata = CommitMetadata {
                    date: formatted_date,
                    description: description.to_owned(),
                    commit_id: commit_id.clone(),
                    pointer_to_data: 0,
                    size: file_contents.len() as i32,
                };
                let metadata_string = serde_json::to_string(&vec![metadata])?;
                metadata_file.write_all(metadata_string.as_bytes())?;

                let mut file = File::create(last_committed_file_data_path.clone())?;
                file.write_all(file_contents.as_bytes())?;

                continue;
            }
        };
        let mut file_metadata: Vec<CommitMetadata> = serde_json::from_str(&file_metadata_string)?;
        let last_committed_metadata = file_metadata.last().unwrap();
        let last_committed_file_pointer = last_committed_metadata.pointer_to_data;
        let last_committed_file_size = last_committed_metadata.size;

        let last_committed_file_contents = read_part_of_file(
            &last_committed_file_data_path,
            last_committed_file_pointer as u64,
            last_committed_file_size as usize,
        )?;

        if !compare_strings(&last_committed_file_contents, &file_contents) {
            let mut metadata_file = File::create(last_committed_file_metadata_path.clone())?;
            let mut file = OpenOptions::new()
                .append(true)
                .open(last_committed_file_data_path.clone())?;
            let metadata = CommitMetadata {
                date: formatted_date,
                description: description.to_owned(),
                commit_id: commit_id.clone(),
                pointer_to_data: last_committed_file_pointer + last_committed_file_size,
                size: file_contents.len() as i32,
            };
            file_metadata.push(metadata);

            let metadata_string = serde_json::to_string(&file_metadata)?;
            metadata_file.write_all(metadata_string.as_bytes())?;
            file.write_all(file_contents.as_bytes())?;
        }
    }

    Ok(())
}
