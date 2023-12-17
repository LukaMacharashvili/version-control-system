use aws_sdk_s3 as s3;
use s3::Client;
use std::fs;
use std::fs::File;
use std::io::Write;

use crate::utils::dates::get_current_formatted_date;
use crate::utils::fs_provider::*;
use crate::utils::s3_provider::*;
use crate::utils::*;

pub fn init() -> std::io::Result<()> {
    let history_path = ".history".to_owned();
    check_if_initialized()?;

    fs::create_dir(&history_path)?;
    File::create(history_path + "/commits.json")?.write_all(b"[]")?;

    Ok(())
}

pub fn commit(description: &str) -> std::io::Result<()> {
    check_if_initialized()?;

    let history_path = ".history".to_owned();
    let commit_id: String = generate_commit_id();
    let ignores = load_ignores();
    let file_paths = traverse_directory(None, Some(&ignores));
    let formatted_date = get_current_formatted_date();

    add_commit(
        &(history_path.clone() + "/commits.json"),
        Commit {
            date: formatted_date.clone(),
            description: description.to_owned(),
            commit_id: commit_id.clone(),
        },
    )?;

    for file_path in file_paths {
        let file_name_for_history = file_path.clone().to_str().unwrap().replace("/", "_");
        let last_committed_file_path = history_path.clone() + &"/" + file_name_for_history.as_str();
        let file_contents = fs::read_to_string(file_path.clone())?;

        let file_metadata_string_result: Result<Vec<CommitMetadata>, std::io::Error> =
            file_metadata(&last_committed_file_path);
        let mut file_metadata = match file_metadata_string_result {
            Ok(res) => res,
            Err(_) => {
                fs::create_dir(&last_committed_file_path)?;
                write_to_metadata_file(
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
        let last_committed_metadata = file_metadata.last().unwrap();
        let last_committed_file_pointer = last_committed_metadata.pointer_to_data;
        let last_committed_file_size = last_committed_metadata.size;

        let last_committed_file_contents = read_part_of_file(
            &(last_committed_file_path.clone() + "/data.bin"),
            last_committed_file_pointer as u64,
            last_committed_file_size as usize,
        )?;

        if &last_committed_file_contents == &file_contents {
            file_metadata.push(CommitMetadata {
                date: formatted_date.clone(),
                description: description.to_owned(),
                commit_id: commit_id.clone(),
                pointer_to_data: last_committed_file_pointer + last_committed_file_size,
                size: file_contents.len() as i32,
            });

            write_to_metadata_file(&last_committed_file_path, file_metadata)?;
            write_to_data_file(&last_committed_file_path, &file_contents, true)?;
        }
    }

    Ok(())
}

pub fn commits() -> std::io::Result<()> {
    check_if_initialized()?;
    let commits = load_commits("/commits.json")?;

    for commit in commits {
        println!(
            "{} {} {}",
            commit.date, commit.commit_id, commit.description
        );
    }

    Ok(())
}

pub fn view(branch_id: &str) -> std::io::Result<()> {
    check_if_initialized()?;
    let ignores = load_ignores();
    delete_contents_of_directory(".", Some(&ignores))?;
    load_commit(branch_id)?;

    Ok(())
}

pub fn set_remote(bucket_name: &str) -> std::io::Result<()> {
    if File::open(".history/remote").is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Remote already set",
        ));
    }
    File::create(".history/remote")?.write_all(bucket_name.as_bytes())
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

    let commits = load_commits("/commits.json")?;
    let last_commit = commits.last().unwrap();
    let last_commit_id = last_commit.commit_id.clone();
    load_commit(&last_commit_id)?;

    Ok(())
}

pub async fn pull(client: &Client) -> std::io::Result<()> {
    let bucket_name = fs::read_to_string(".history/remote")?;
    let ignores = load_ignores();
    delete_contents_of_directory(".", Some(&ignores))?;
    clone(client, &bucket_name).await?;

    Ok(())
}

pub async fn push(client: &Client) -> std::io::Result<()> {
    let bucket_name = fs::read_to_string(".history/remote")?;
    create_file_from_s3object(
        client,
        ".history/temp-commits.json",
        &bucket_name,
        ".history/commits.json",
    )
    .await?;

    let remote_commits = load_commits("/temp-commits.json")?;
    let local_commits = load_commits("/commits.json")?;
    fs::remove_file(".history/temp-commits.json")?;

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

    sync_local_history_with_s3(client, &bucket_name, ".history").await?;

    Ok(())
}
