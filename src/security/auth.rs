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
use log::{debug, info};
use std::collections::HashMap;
use std::rc::Rc;
use std::task::{Context, Poll}; // Importation des macros de logging

pub struct AuthMiddleware {
    htpasswd: Rc<HashMap<String, String>>,
}

impl AuthMiddleware {
    pub fn new(htpasswd: HashMap<String, String>) -> Self {
        Self {
            htpasswd: Rc::new(htpasswd),
        }
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
            htpasswd: Rc::clone(&self.htpasswd),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    htpasswd: Rc<HashMap<String, String>>,
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
        let htpasswd = Rc::clone(&self.htpasswd);

        Box::pin(async move {
            debug!("Processing request: {}", req.path());
            if let Some(auth_header) = req.headers().get("Authorization") {
                info!("Authorization header found");
                if let Ok(auth_str) = auth_header.to_str() {
                    let parts: Vec<&str> = auth_str.split_whitespace().collect();
                    if parts.len() == 2 && parts[0] == "Basic" {
                        if let Ok(decoded) = STANDARD.decode(parts[1]) {
                            info!("Successfully decoded Authorization header");
                            if let Ok(credentials) = String::from_utf8(decoded) {
                                let mut split = credentials.splitn(2, ':');
                                let username = split.next().unwrap_or("");
                                let password = split.next().unwrap_or("");

                                info!("Attempting to authenticate user");
                                if let Some(hashed) = htpasswd.get(username) {
                                    if Htpasswd::new_owned(&format!("{}:{}", username, hashed))
                                        .check(username, password)
                                    {
                                        info!("User authenticated successfully");
                                        let res = service.call(req).await?;
                                        return Ok(res.map_into_left_body());
                                    } else {
                                        info!("Invalid password for user");
                                    }
                                } else {
                                    info!("User not found in htpasswd");
                                }
                            } else {
                                debug!("Failed to convert decoded credentials to UTF-8");
                            }
                        } else {
                            debug!("Failed to decode Authorization header");
                        }
                    } else {
                        debug!("Authorization header format is invalid");
                    }
                } else {
                    debug!("Failed to parse Authorization header");
                }
            } else {
                info!("No Authorization header found");
            }

            let response = req.into_response(
                HttpResponse::Unauthorized()
                    .append_header(("WWW-Authenticate", r#"Basic realm=\"Restricted\""#))
                    .finish()
                    .map_into_right_body(),
            );
            info!("Request unauthorized, returning 401");
            Ok(response)
        })
    }
}
