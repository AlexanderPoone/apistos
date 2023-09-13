use actix_web::dev::Payload;
use actix_web::http::Error;
use actix_web::{FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use netwopenapi::ApiSecurity;

#[derive(ApiSecurity)]
#[openapi_security(scheme(security_type(api_key(name = "api_key", api_key_in = "header"))))]
pub struct ApiKey;

impl FromRequest for ApiKey {
  type Error = Error;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(_req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
    ready(Ok(ApiKey {}))
  }
}
