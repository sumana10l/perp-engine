use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use crate::auth::verify_token;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService { service })
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.path() == "/health" || req.path() == "/auth/login" {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        match token {
            Some(t) => match verify_token(t) {
                Ok(_) => {
                    let fut = self.service.call(req);
                    Box::pin(async move { fut.await })
                }
                Err(_) => Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                }),
            },
            None => Box::pin(async {
                Err(actix_web::error::ErrorUnauthorized("Missing token"))
            }),
        }
    }
}