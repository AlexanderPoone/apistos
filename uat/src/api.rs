use actix_web::web::{Json, Path};
use actix_web::Error;
use apistos::actix::CreatedJson;
use apistos::{api_operation, ApiComponent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// JsonSchema !
#[derive(Deserialize, JsonSchema, ApiComponent)]          // <------------- `JsonSchema`, `ApiComponent` are Apistos Traits
pub struct NewTodo {
  pub title: String,
  pub description: Option<String>,
}

// JsonSchema !
#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct Todo {
  pub id: Uuid,
  pub title: String,
  pub description: Option<String>,
}

// #[api_operation(
//   tag = "pet",
//   summary = "Add a new pet to the store",
//   description = r###"Add a new pet to the store
//     Plop"###,
//   error_code = 405
// )]
#[api_operation(summary = "Get an element from the todo list")]
pub(crate) async fn get_todo(todo_id: Path<Uuid>) -> Result<Json<Todo>, Error> {          // <----- scope("/test").service(scope("/todo").service(resource("/{todo_id}") ... the param is defined as a Uuid
  // because it is a sample app, we do not currently implement any logic to store todos
  Ok(Json(Todo {
    id: todo_id.into_inner(),
    title: "some title".to_string(),
    description: None,
  }))
}
// Use `String` for string param
//
// // extract path info from "/{name}/{count}/index.html" into tuple
// // {name}  - deserialize a String
// // {count} - deserialize a u32
// #[get("/{name}/{count}/index.html")]
// async fn index(path: web::Path<(String, u32)>) -> String {

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn add_todo(body: Json<NewTodo>) -> Result<CreatedJson<Todo>, Error> {   // <----- it should eat both a `Path` (for parameters) and a `Json` (for body)
  let new_todo = body.into_inner();
  Ok(CreatedJson(Todo {
    id: Uuid::new_v4(),
    title: new_todo.title,
    description: new_todo.description,
  }))
}