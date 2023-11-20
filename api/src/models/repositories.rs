use mysql::prelude::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Repository {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub remote_url: String,
    pub username: String,
}
