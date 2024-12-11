use crate::api::routes::routes;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use apistos::app::{BuildConfig, OpenApiWrapper};
use apistos::{RapidocConfig, RedocConfig, ScalarConfig, SwaggerUIConfig};
use apistos::info::{Contact, Info, License};
use apistos::paths::ExternalDocumentation;
use apistos::server::Server;
use apistos::spec::Spec;
use apistos::tag::Tag;
use apistos::web::scope;
use std::{env, fs};
use std::error::Error;
use std::net::Ipv4Addr;

mod api;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
  env_logger::init();

  let data = "埗鰂";
  fs::write("C:/Users/Alex/Desktop/a.txt", data).expect("Unable to write file");

  let core_port = match env::var_os("ASPNETCORE_PORT") {
    Some(v) => v.into_string().unwrap().parse::<u16>().unwrap(),
    None => panic!("$ASPNETCORE_PORT is not set")
  };

  let data2 = "Some data!";
  fs::write("C:/Users/Alex/Desktop/b.txt", data2).expect("Unable to write file");

  HttpServer::new(move || {
    let spec = Spec {   // <--------------------------- SPEC is the main object
      default_tags: vec!["api".to_owned()],
      tags: vec![
        Tag {
          name: "api".to_string(),
          description: Some("Everything about petstore".to_string()),
          ..Default::default()
        },
        Tag {
          name: "pet".to_string(),
          description: Some("Everything about your Pets".to_string()),
          ..Default::default()
        },
        Tag {
          name: "store".to_string(),
          description: Some("Access to Petstore orders".to_string()),
          ..Default::default()
        },
        Tag {
          name: "user".to_string(),
          description: Some("Operations about user".to_string()),
          ..Default::default()
        },
      ],
      info: Info {    // <------------------------------------ INFO inside SPEC
        title: "Swagger Petstore - OpenAPI 3.0".to_string(),
        description: Some("This is a sample Pet Store Server based on the OpenAPI 3.0 specification.  You can find out more about\nSwagger at [http://swagger.io](http://swagger.io). In the third iteration of the pet store, we've switched to the design first approach!\nYou can now help us improve the API whether it's by making changes to the definition itself or to the code.\nThat way, with time, we can improve the API in general, and expose some of the new features in OAS3.\n\nSome useful links:\n- [The Pet Store repository](https://github.com/swagger-api/swagger-petstore)\n- [The source API definition for the Pet Store](https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml)".to_string()),
        terms_of_service: Some("http://swagger.io/terms/".to_string()),
        contact: Some(Contact {
          email: Some("apiteam@swagger.io".to_string()),
          ..Default::default()
        }),
        license: Some(License {
          name: "Apache 2.0".to_string(),
          url: Some("http://www.apache.org/licenses/LICENSE-2.0.html".to_string()),
          ..Default::default()
        }),
        version: "1.0.17".to_string(),
        ..Default::default()
      },
      external_docs: Some(ExternalDocumentation {
        description: Some("Find out more about Swagger".to_string()),
        url: "http://swagger.io".to_string(),
        ..Default::default()
      }),
      servers: vec![Server { url: "/api/v3".to_string(), ..Default::default() }],
      ..Default::default()
    };

    let data3 = "Some data!";
    fs::write("C:/Users/Alex/Desktop/c.txt", data3).expect("Unable to write file");

    App::new()
      .document(spec)  // <------------------------------------ document(spec)
      .wrap(Logger::default())  // <--------------------------- As usual from here.
      .service(scope("/test").service(routes()))
      .build_with(
        "/openapi.json",
        BuildConfig::default()   // <-------------------------- BuildConfig is apistos
          .with(RapidocConfig::new(&"/rapidoc"))
          .with(RedocConfig::new(&"/redoc"))    // Redoc has no Try feature?
          .with(ScalarConfig::new(&"/scalar"))
          .with(SwaggerUIConfig::new(&"/swagger")),
      )
  })
    .bind((Ipv4Addr::UNSPECIFIED, core_port))?
    .run()
    .await

  // cf. Rocket
  // let core_port = match env::var_os("ASPNETCORE_PORT") {
  //     Some(v) => v.into_string().unwrap().parse::<i32>().unwrap(),
  //     None => panic!("$ASPNETCORE_PORT is not set")
  // };

  // let cfg = Config::figment()
  //     .merge(("address", "0.0.0.0"))
  //     .merge(("port", core_port));

  // rocket::custom(cfg)
  //     .manage(db)
  //     .manage(channel::<Message>(1024).0)
  //     .mount("/", routes![post, events])
  //     .mount("/", FileServer::from(relative!("static")))
  //     .attach(CORS)
}
