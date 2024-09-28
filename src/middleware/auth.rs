use std::{
    future::{ready, Future, Ready},
    pin::Pin,
};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{config::Config, models::auth::JWTClaims};

const UNAUTHORIZED_MESSAGE: &str = r#"{{"status": "error", "message": "Unauthorized"}}"#;

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

type LocalBoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let authorization = req.headers().get("Authorization");

        let bearer_token = match authorization.and_then(|header| header.to_str().ok()) {
            Some(auth_str) if auth_str.starts_with("Bearer ") => auth_str[7..].to_string(),
            _ => {
                return Box::pin(async move { Err(ErrorUnauthorized(UNAUTHORIZED_MESSAGE)) });
            }
        };

        let jwt_secret = Config::get_env_param("JWT_SECRET");
        let token = match decode::<JWTClaims>(
            &bearer_token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token) => token,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized(UNAUTHORIZED_MESSAGE))
                });
            }
        };

        req.extensions_mut().insert(token.claims.sub);

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
