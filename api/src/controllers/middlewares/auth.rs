use actix_web::dev::{self};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::LocalBoxFuture;
use jsonwebtokens_cognito::KeySet;
use std::env;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::str::FromStr;

pub struct ExtractAuth;

impl<S: 'static, B> Transform<S, ServiceRequest> for ExtractAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ExtractAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ExtractAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ExtractAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ExtractAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        let authorization = req
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        let token = authorization.replace("Bearer ", "");

        let region = env::var("AWS_REGION").unwrap();
        let user_pool_id = env::var("COGNITO_USER_POOL_ID").unwrap();
        let pool_client_id = env::var("COGNITO_CLIENT_ID").unwrap();

        let keyset = KeySet::new(region, user_pool_id).unwrap();
        let verifier = keyset
            .new_id_token_verifier(&[&pool_client_id])
            .build()
            .unwrap();
        Box::pin(async move {
            let claims = keyset.verify(&token, &verifier).await.unwrap();
            let username = claims.get("cognito:username").unwrap();
            req.headers_mut().append(
                HeaderName::from_str("username").unwrap(),
                HeaderValue::from_str(&username.to_string()).unwrap(),
            );
            svc.call(req).await
        })
    }
}
