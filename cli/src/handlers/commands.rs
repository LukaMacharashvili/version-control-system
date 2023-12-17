use aws_sdk_s3 as s3;
use s3::Client;
use std::fs;
use std::fs::File;
use std::io::Write;

use crate::utils::constants::{
    MAIN_COMMITS_METADATA_FILE_PATH, REMOTE_REPOSITORY_REFERENCE_FILE_PATH, VSM_DIR,
};
use crate::utils::dates::get_current_formatted_date;
use crate::utils::fs_provider::*;
use crate::utils::s3_provider::*;
use crate::utils::types::Commit;
use crate::utils::types::CommitMetadata;
use crate::utils::*;

pub fn init() -> std::io::Result<()> {
    check_if_initialized()?;

    fs::create_dir(VSM_DIR)?;
    File::create(MAIN_COMMITS_METADATA_FILE_PATH)?.write_all(b"[]")?;

    Ok(())
}

pub fn commit(description: &str) -> std::io::Result<()> {
    check_if_initialized()?;

    let commit_id: String = generate_commit_id();
    let files_to_ignore = list_files_ignore();
    let file_paths = get_file_paths_recursively(None, Some(&files_to_ignore));
    let formatted_date = get_current_formatted_date();

    add_root_commit_metadata(Commit {
        date: formatted_date.clone(),
        description: description.to_owned(),
        commit_id: commit_id.clone(),
    })?;

    for file_path in file_paths {
        let file_name_for_history = file_path.clone().to_str().unwrap().replace("/", "_");
        let last_committed_file_path = VSM_DIR.to_owned() + "/" + file_name_for_history.as_str();
        let file_contents = fs::read_to_string(file_path.clone())?;

        let file_metadata_string_result: Result<Vec<CommitMetadata>, std::io::Error> =
            commits_metadata(&last_committed_file_path);
        let mut commits_metadata = match file_metadata_string_result {
            Ok(res) => res,
            Err(_) => {
                fs::create_dir(&last_committed_file_path)?;
                write_to_commit_metadata_file(
                    &last_committed_file_path,
                    vec![CommitMetadata {
                        date: formatted_date.clone(),
                        description: description.to_owned(),
                        commit_id: commit_id.clone(),
                        pointer_to_data: 0,
                        size: file_contents.len() as i32,
                    }],
                )?;

                write_to_data_file(&last_committed_file_path, &file_contents, false)?;
                continue;
            }
        };
        let last_committed_metadata = commits_metadata.last().unwrap();
        let last_committed_file_pointer = last_committed_metadata.pointer_to_data;
        let last_committed_file_size = last_committed_metadata.size;

        let last_committed_file_contents = read_part_of_file(
            &(last_committed_file_path.clone() + "/data.bin"),
            last_committed_file_pointer as u64,
            last_committed_file_size as usize,
        )?;

        if &last_committed_file_contents == &file_contents {
            commits_metadata.push(CommitMetadata {
                date: formatted_date.clone(),
                description: description.to_owned(),
                commit_id: commit_id.clone(),
                pointer_to_data: last_committed_file_pointer + last_committed_file_size,
                size: file_contents.len() as i32,
            });

            write_to_commit_metadata_file(&last_committed_file_path, commits_metadata)?;
            write_to_data_file(&last_committed_file_path, &file_contents, true)?;
        }
    }

    Ok(())
}

pub fn view(branch_id: &str) -> std::io::Result<()> {
    check_if_initialized()?;

    let files_to_ignore = list_files_ignore();
    delete_contents_of_directory(".", Some(&files_to_ignore))?;
    load_commit(branch_id)?;

    Ok(())
}

pub fn set_remote(bucket_name: &str) -> std::io::Result<()> {
    if File::open(REMOTE_REPOSITORY_REFERENCE_FILE_PATH).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Remote already set",
        ));
    }
    File::create(REMOTE_REPOSITORY_REFERENCE_FILE_PATH)?.write_all(bucket_name.as_bytes())
}

pub async fn clone(client: &Client, bucket_name: &str) -> std::io::Result<()> {
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .send()
        .await
        .unwrap();

    for obj in objects.contents() {
        create_file_from_s3object(
            client,
            &obj.key().unwrap(),
            bucket_name,
            &obj.key().unwrap(),
        )
        .await?;
    }

    let commits = list_commits(MAIN_COMMITS_METADATA_FILE_PATH)?;
    let last_commit = commits.last().unwrap();
    let last_commit_id = last_commit.commit_id.clone();
    load_commit(&last_commit_id)?;

    Ok(())
}

pub async fn pull(client: &Client) -> std::io::Result<()> {
    let bucket_name = fs::read_to_string(REMOTE_REPOSITORY_REFERENCE_FILE_PATH)?;
    let files_to_ignore = list_files_ignore();
    delete_contents_of_directory(".", Some(&files_to_ignore))?;
    clone(client, &bucket_name).await?;

    Ok(())
}

pub async fn push(client: &Client) -> std::io::Result<()> {
    let bucket_name = fs::read_to_string(REMOTE_REPOSITORY_REFERENCE_FILE_PATH)?;
    create_file_from_s3object(
        client,
        &(MAIN_COMMITS_METADATA_FILE_PATH.to_owned() + "-temp"),
        &bucket_name,
        MAIN_COMMITS_METADATA_FILE_PATH,
    )
    .await?;

    let remote_commits = list_commits(&(MAIN_COMMITS_METADATA_FILE_PATH.to_owned() + "-temp"))?;
    let local_commits = list_commits(MAIN_COMMITS_METADATA_FILE_PATH)?;
    fs::remove_file(MAIN_COMMITS_METADATA_FILE_PATH.to_owned() + "-temp")?;

    let last_remote_commit = remote_commits.last().unwrap();
    let last_remote_commit_id = last_remote_commit.commit_id.clone();

    let found_commit = find_commit_by_commit_id(&local_commits, &last_remote_commit_id);
    if found_commit.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Remote history is ahead of local history, pull the changes first",
        ));
    }

    if remote_commits.len() >= local_commits.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Nothing to push",
        ));
    }

    sync_local_history_with_s3(client, &bucket_name, VSM_DIR).await?;

    Ok(())
}
