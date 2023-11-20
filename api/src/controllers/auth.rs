use std::env;

use crate::models::user::User;
use actix_web::{post, web, HttpResponse};
use aws_sdk_cognitoidentityprovider::{types::AuthFlowType, Client as CognitoClient};

#[post("/register")]
pub async fn register(auth_db: web::Data<CognitoClient>, user: web::Json<User>) -> HttpResponse {
    let user_client_id: String = env::var("COGNITO_CLIENT_ID").expect("COGNITO_CLIENT_ID not set");
    let user = user.into_inner();
    let username = user.username;
    let password = user.password;

    let register_res = auth_db
        .sign_up()
        .client_id(user_client_id)
        .username(username)
        .password(password)
        .send()
        .await;

    println!("Register: {:?}", register_res);
    match register_res {
        Ok(_) => HttpResponse::Ok().body("User successfully registered"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/login")]
pub async fn login(auth_db: web::Data<CognitoClient>, user: web::Json<User>) -> HttpResponse {
    let user_client_id: String = env::var("COGNITO_CLIENT_ID").expect("COGNITO_CLIENT_ID not set");
    let user = user.into_inner();
    let auth_res = auth_db
        .initiate_auth()
        .client_id(user_client_id)
        .auth_flow(AuthFlowType::UserPasswordAuth)
        .auth_parameters("USERNAME", user.username)
        .auth_parameters("PASSWORD", user.password)
        .send()
        .await;

    println!("Login: {:?}", auth_res);
    match auth_res {
        Ok(res) => {
            let id_token = res.authentication_result().unwrap().id_token().unwrap();
            HttpResponse::Ok().body(id_token.to_owned())
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}
