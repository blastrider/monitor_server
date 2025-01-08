use actix_service::Transform;
use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use futures_util::future::{ok, LocalBoxFuture, Ready};
use htpasswd_verify::Htpasswd;
use std::collections::HashMap;
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct AuthMiddleware {
    htpasswd: HashMap<String, String>,
}

impl AuthMiddleware {
    pub fn new(htpasswd: HashMap<String, String>) -> Self {
        Self { htpasswd }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            htpasswd: self.htpasswd.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    htpasswd: HashMap<String, String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let htpasswd = self.htpasswd.clone();

        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    let parts: Vec<&str> = auth_str.split_whitespace().collect();
                    if parts.len() == 2 && parts[0] == "Basic" {
                        if let Ok(decoded) = STANDARD.decode(parts[1]) {
                            // Utilisation du moteur STANDARD pour décoder
                            if let Ok(credentials) = String::from_utf8(decoded) {
                                let mut split = credentials.splitn(2, ':');
                                let username = split.next().unwrap_or("");
                                let password = split.next().unwrap_or("");

                                if let Some(hashed) = htpasswd.get(username) {
                                    if Htpasswd::new_owned(&format!("{}:{}", username, hashed))
                                        .check(username, password)
                                    {
                                        let res = service.call(req).await?;
                                        return Ok(res.map_into_left_body());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let response = req.into_response(
                HttpResponse::Unauthorized()
                    .append_header(("WWW-Authenticate", r#"Basic realm="Restricted""#))
                    .finish()
                    .map_into_right_body(),
            );
            Ok(response)
        })
    }
}
