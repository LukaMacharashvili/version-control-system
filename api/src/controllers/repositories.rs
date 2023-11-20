use crate::{models::repositories::Repository, repositories::repositories as model};
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use mysql::{serde_json, Pool};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepositoryCreateDto {
    pub name: String,
    pub description: String,
    pub remote_url: String,
}

#[get("/")]
pub async fn list(conn: web::Data<Pool>, req: HttpRequest) -> HttpResponse {
    let username = req.headers().get("username").unwrap().to_str().unwrap();
    let list = model::list(&conn, username).unwrap();
    HttpResponse::Ok().json(serde_json::to_string(&list).unwrap())
}

#[get("/:id")]
pub async fn fetch(conn: web::Data<Pool>, id: web::Path<String>) -> HttpResponse {
    let repository = model::fetch(&conn, &id).unwrap();
    HttpResponse::Ok().json(serde_json::to_string(&repository).unwrap())
}

#[post("/")]
pub async fn create(
    conn: web::Data<Pool>,
    repository_dto: web::Json<RepositoryCreateDto>,
    req: HttpRequest,
) -> HttpResponse {
    let username = req.headers().get("username").unwrap().to_str().unwrap();
    let repository = Repository {
        id: None,
        name: repository_dto.name.to_owned(),
        description: repository_dto.description.to_owned(),
        remote_url: repository_dto.remote_url.to_owned(),
        username: username.to_owned(),
    };

    match model::create(&conn, repository) {
        Ok(_) => HttpResponse::Ok().body("Repository created!"),
        Err(_) => HttpResponse::InternalServerError().body("Error trying to create repository!"),
    }
}

#[put("/:id")]
pub async fn update(
    conn: web::Data<Pool>,
    repository_dto: web::Json<RepositoryCreateDto>,
    req: HttpRequest,
    id: web::Path<String>,
) -> HttpResponse {
    let username = req.headers().get("username").unwrap().to_str().unwrap();
    let repository = Repository {
        id: None,
        name: repository_dto.name.to_owned(),
        description: repository_dto.description.to_owned(),
        remote_url: repository_dto.remote_url.to_owned(),
        username: username.to_owned(),
    };

    let found_repository = model::fetch(&conn, &id).unwrap().unwrap();
    if found_repository.username != username {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    match model::update(&conn, &id, repository) {
        Ok(_) => HttpResponse::Ok().body("Repository updated!"),
        Err(_) => HttpResponse::InternalServerError().body("Error trying to update repository!"),
    }
}

#[delete("/:id")]
pub async fn delete(
    conn: web::Data<Pool>,
    id: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let username = req.headers().get("username").unwrap().to_str().unwrap();
    let repository = model::fetch(&conn, &id).unwrap().unwrap();
    if repository.username != username {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    model::delete(&conn, &id).unwrap();
    HttpResponse::Ok().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create)
        .service(fetch)
        .service(list)
        .service(update)
        .service(delete);
}
