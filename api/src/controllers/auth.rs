use crate::models::user::User;
use actix_web::{post, web, HttpRequest, HttpResponse};
use aws_sdk_cognitoidentityprovider::{types::AuthFlowType, Client as CognitoClient};

#[post("/register")]
pub async fn register(authDb: web::Data<CognitoClient>, user: web::Json<User>) -> HttpResponse {
    let user = user.into_inner();
    let username = user.username;
    let password = user.password;

    let user = authDb
        .sign_up()
        .username(username)
        .password(password)
        .send()
        .await;
    match user {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/login")]
pub async fn login(authDb: web::Data<CognitoClient>, req: HttpRequest) -> HttpResponse {
    let username = req
        .headers()
        .get("username")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let password = req
        .headers()
        .get("password")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let user = authDb
        .initiate_auth()
        .auth_flow(AuthFlowType::UserPasswordAuth)
        .auth_parameters("USERNAME", username)
        .auth_parameters("PASSWORD", password)
        .send()
        .await;

    match user {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}
