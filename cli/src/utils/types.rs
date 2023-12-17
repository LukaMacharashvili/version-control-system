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
