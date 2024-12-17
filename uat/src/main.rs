use crate::api::*;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use apistos::app::{BuildConfig, OpenApiWrapper};
use apistos::info::Info;
use apistos::server::Server;
use apistos::spec::Spec;
use apistos::web::{get, post, resource, scope};
use apistos::{RapidocConfig, RedocConfig, ScalarConfig, SwaggerUIConfig};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::error::Error;
use std::net::Ipv4Addr;

mod api;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
  dotenv().ok();
  
  let core_port = match env::var_os("ASPNETCORE_PORT") {
    Some(v) => v.into_string().unwrap().parse::<u16>().unwrap(),
    None => panic!("$ASPNETCORE_PORT is not set")
  };
  
  // Database connection pool
  let database_url = env::var("DATABASE_URL").expect("POSTGRES_PW must be set");  
  let pool = PgPool::connect(&database_url).await.unwrap();

  HttpServer::new(move || {
    let spec = Spec { // <------------------ Again, SPEC
      info: Info {   // <------------------------- INFO inside SPEC
        title: "A well documented API".to_string(),
        description: Some(
          "This is an API documented using Apistos,\na wonderful new tool to document your actix API !".to_string(), // `Option` means `Some(...)``
        ),
        ..Default::default()  // <---------------- Use defaults for the rest
      },
      servers: vec![Server {
        url: "/api/v3".to_string(),
        ..Default::default() // <---------------- Use defaults for the rest
      }],
      ..Default::default()   // <---------------- Use defaults for the rest
    };

    App::new()
    .document(spec) // <-------------------------- document(spec), how can they do this? They use traits...
    .wrap(Logger::default()) // <----------------- As usual from here.
    .app_data(web::Data::new(pool.clone()))
    .service(resource("/").route(get().to(testConnection))) // <---------------------- {param} and get()/post() are defined here. EACH ENDPOINT IS ONE **SERVICE**.
    .service(resource("/signIn_github").route(post().to(signIn_github)))
    .service(resource("/signIn_google").route(post().to(signIn_google)))
    .service(resource("/signUp").route(post().to(signUp)))
    .service(resource("/confirmAccount").route(post().to(confirmAccount)))
    .service(resource("/signIn").route(post().to(signIn)))
    .service(resource("/signOut").route(post().to(signOut)))
    .service(resource("/requestChangePassword").route(post().to(requestChangePassword)))
    .service(resource("/changePasswordWithToken").route(post().to(changePasswordWithToken)))
    .service(resource("/setAccountSetting").route(post().to(setAccountSetting)))
    .service(resource("/getBlogs").route(get().to(getBlogs)))
    .service(resource("/getProjects").route(get().to(getProjects)))
    .service(resource("/converted").route(post().to(converted)))
    .build_with(
      "/openapi.json",
      BuildConfig::default()
        .with(RapidocConfig::new(&"/rapidoc"))
        .with(RedocConfig::new(&"/redoc"))
        .with(ScalarConfig::new(&"/scalar"))
        .with(SwaggerUIConfig::new(&"/swagger")),
    )
  })
  .bind((Ipv4Addr::UNSPECIFIED, core_port))?
  .run()
  .await
}