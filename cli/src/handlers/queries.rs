use crate::utils::constants::MAIN_COMMITS_METADATA_FILE_PATH;
use crate::utils::*;

pub fn log_commits() -> std::io::Result<()> {
    check_if_initialized()?;
    let commits = list_commits(MAIN_COMMITS_METADATA_FILE_PATH)?;

    for commit in commits {
        println!(
            "{} {} {}",
            commit.date, commit.commit_id, commit.description
        );
    }

    Ok(())
}
