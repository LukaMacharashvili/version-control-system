use actix_web::web;
use mysql::{params, prelude::Queryable, Pool};

use crate::models::repositories::Repository;

pub fn list(conn: &web::Data<Pool>, username: &str) -> Result<Vec<Repository>, mysql::Error> {
    let mut conn = conn.get_ref().get_conn().unwrap();

    let repositories = conn.exec::<Repository, _, _>(
        r"SELECT id, name, description, remote_url, username FROM repositories WHERE username = :username",
        params! {
            "username" => username,
        },
    )?;

    Ok(repositories)
}

pub fn fetch(conn: &web::Data<Pool>, id: &str) -> Result<Option<Repository>, mysql::Error> {
    let mut conn = conn.get_ref().get_conn().unwrap();

    let repository = conn.exec_first::<Repository, _, _>(
        r"SELECT id, name, description, remote_url, username FROM repositories WHERE id = :id",
        params! {
            "id" => id,
        },
    )?;

    Ok(repository)
}

pub fn create(conn: &web::Data<Pool>, repository: Repository) -> Result<(), mysql::Error> {
    let mut conn = conn.get_ref().get_conn().unwrap();

    conn.exec_drop(
        r"INSERT INTO repositories (name, description, remote_url, username)
          VALUES (:name, :description, :remote_url, :username)",
        params! {
            "name" => repository.name,
            "description" => repository.description,
            "remote_url" => repository.remote_url,
            "username" => repository.username,
        },
    )
}

pub fn update(
    conn: &web::Data<Pool>,
    id: &str,
    repository: Repository,
) -> Result<(), mysql::Error> {
    let mut conn = conn.get_ref().get_conn().unwrap();

    conn.exec_drop(
        r"UPDATE repositories SET name = :name, description = :description, remote_url = :remote_url WHERE id = :id",
        params! {
            "id" => id,
            "name" => repository.name,
            "description" => repository.description,
            "remote_url" => repository.remote_url,
        },
    )
}

pub fn delete(conn: &web::Data<Pool>, id: &str) -> Result<(), mysql::Error> {
    let mut conn = conn.get_ref().get_conn().unwrap();

    conn.exec_drop(
        r"DELETE FROM repositories WHERE id = :id",
        params! {
            "id" => id,
        },
    )
}
