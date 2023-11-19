mod auth;
mod repositories;
use actix_web::{web, App, HttpServer};
use aws_sdk_cognitoidentityprovider::Client as CognitoClient;

pub fn config(cognito_client: CognitoClient, db: web::Data<DbPool>) {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(cognito_client.clone()))
            .service(web::scope("/auth").configure(auth::config))
            .service(web::scope("/repositories").configure(repositories::config))
    });
}
