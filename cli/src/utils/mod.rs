pub mod constants;
pub mod dates;
pub mod fs_provider;
pub mod s3_provider;
pub mod types;

use self::constants::{
    COMMIT_METADATA_RELATIVE_PATH, IGNORE_FILES_PATH, MAIN_COMMITS_METADATA_FILE_PATH,
};
use self::fs_provider::{get_file_paths_recursively, read_part_of_file};
use self::types::{Commit, CommitMetadata};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs::File;
use std::fs::{self};
use std::io::{self, Read, Write};
use std::path::Path;
use std::thread;

pub fn generate_commit_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

pub fn check_if_initialized() -> std::io::Result<()> {
    let history_path = ".history".to_owned();
    let git_path = ".git".to_owned();
    if fs::read_dir(git_path.clone()).is_ok() || fs::read_dir(history_path.clone()).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Already initialized",
        ));
    }

    Ok(())
}

pub fn commits_metadata(path: &str) -> std::io::Result<Vec<CommitMetadata>> {
    let file_metadata_string_result =
        fs::read_to_string(path.to_owned() + COMMIT_METADATA_RELATIVE_PATH)?;
    serde_json::from_str(&file_metadata_string_result).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
}

pub fn list_commits(path: &str) -> std::io::Result<Vec<Commit>> {
    let commits_string_result = fs::read_to_string(path)?;
    serde_json::from_str::<Vec<Commit>>(&commits_string_result).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })
}

pub fn add_root_commit_metadata(commit_metadata: Commit) -> std::io::Result<()> {
    let mut commits_metadata = list_commits(MAIN_COMMITS_METADATA_FILE_PATH)?;
    commits_metadata.push(commit_metadata);

    let commits_metadata_string = serde_json::to_string(&commits_metadata).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse metadata: {}", e),
        )
    })?;

    File::create(MAIN_COMMITS_METADATA_FILE_PATH)?.write_all(commits_metadata_string.as_bytes())?;

    Ok(())
}

pub fn load_commit(branch_id: &str) -> std::io::Result<()> {
    let history_path = ".history".to_owned();

    if let Ok(entries) = fs::read_dir(history_path.clone()) {
        let mut join_handles = Vec::new();
        for entry in entries {
            let branch_id = branch_id.to_owned();
            let history_path = history_path.clone();
            let handle = thread::spawn(move || {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        return;
                    }
                    let file_name = entry_path.file_name().unwrap().to_str().unwrap().to_owned();
                    let file_name_for_history = file_name.replace("_", "/");
                    let last_committed_file_path: String = history_path.clone() + &"/" + &file_name;

                    let commits_metadata = commits_metadata(&last_committed_file_path).unwrap();
                    let target_commit_metadata_result =
                        find_metadata_by_commit_id(&commits_metadata, &branch_id);
                    let target_commit_metadata = match target_commit_metadata_result {
                        None => commits_metadata.last().unwrap().clone(),
                        _ => target_commit_metadata_result.unwrap(),
                    };

                    let last_committed_file_pointer = target_commit_metadata.pointer_to_data;
                    let last_committed_file_size = target_commit_metadata.size;

                    let last_committed_file_contents = read_part_of_file(
                        &(last_committed_file_path.clone() + "/data.bin"),
                        last_committed_file_pointer as u64,
                        last_committed_file_size as usize,
                    )
                    .unwrap();

                    fs::create_dir_all(Path::new(&file_name_for_history).parent().unwrap())
                        .unwrap();
                    let mut file = File::create(file_name_for_history).unwrap();
                    file.write_all(last_committed_file_contents.as_bytes())
                        .unwrap();
                }
            });

            join_handles.push(handle);
        }

        for handle in join_handles {
            handle.join().unwrap();
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

pub fn find_commit_by_commit_id(metadata: &Vec<Commit>, commit_id: &str) -> Option<Commit> {
    for commit_metadata in metadata {
        if commit_metadata.commit_id == commit_id {
            return Some(commit_metadata.clone());
        }
    }

    None
}

pub fn list_files_ignore() -> Vec<String> {
    let mut ignore_file = match File::open(IGNORE_FILES_PATH) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    let mut ignore_files_string = String::new();
    ignore_file
        .read_to_string(&mut ignore_files_string)
        .unwrap();
    let ignore_file_lines: Vec<&str> = ignore_files_string.split("\n").collect();

    ignore_file_lines
        .iter()
        .filter(|x| !x.starts_with("#") && !x.is_empty())
        .map(|x| x.to_owned().to_owned())
        .collect()
}

pub fn write_to_commit_metadata_file(
    path: &str,
    metadata: Vec<CommitMetadata>,
) -> std::io::Result<()> {
    let mut metadata_file = File::create(path.to_owned() + COMMIT_METADATA_RELATIVE_PATH)?;
    let metadata_string = serde_json::to_string(&metadata)?;
    metadata_file.write_all(metadata_string.as_bytes())?;
    Ok(())
}
