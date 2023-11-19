use actix_web::{post, web, HttpResponse};

#[post("/")]
pub async fn create(db: web::Data<DbPool>) -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create);
}
