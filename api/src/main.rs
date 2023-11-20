mod controllers;
mod models;
mod repositories;
use actix_web::{web, App, HttpServer};
use aws_sdk_cognitoidentityprovider as cognitoidentityprovider;
use mysql::*;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url: &str = &env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool_res = Pool::new(url);
    let pool = match pool_res {
        Ok(p) => p,
        Err(e) => panic!("Error connecting to database: {}", e),
    };
    let cognito_config = aws_config::load_from_env().await;
    let cognito_client = cognitoidentityprovider::Client::new(&cognito_config);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cognito_client.clone()))
            .service(web::scope("/auth").configure(controllers::auth::config))
            .service(
                web::scope("/repositories")
                    .configure(controllers::repositories::config)
                    .wrap(controllers::middlewares::auth::ExtractAuth),
            )
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
