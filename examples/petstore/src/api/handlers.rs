use crate::api::error::ErrorResponse;
use crate::api::models::{Category, Pet, Status, Tag};
use actix_web::web::{Json, Path};
use actix_web::Error;
use netwopenapi::actix::ResponseWrapper;
use netwopenapi::api_operation;
use netwopenapi::path_item_definition::PathItemDefinition;
use netwopenapi::ApiComponent;
use std::collections::BTreeMap;
use std::sync::Arc;
use utoipa::openapi::path::Operation;
use utoipa::openapi::{Components, ComponentsBuilder, PathItem};
use uuid::Uuid;

#[api_operation]
pub(crate) async fn update_pet(
  // Create a new pet in the store
  body: Json<Pet>,
) -> Result<Json<Pet>, Error> {
  Ok(body)
}

// summary Add a new pet to the store
// description Add a new pet to the store
// operationId addPet
#[api_operation]
pub(crate) async fn add_pet(
  // Create a new pet in the store
  body: Json<Pet>,
) -> Result<Json<Pet>, Error> {
  Ok(body)
}

// summary Find pet by ID
// description Returns a single pet
// operationId getPet
#[api_operation]
pub(crate) async fn get_pet(
  // Create a new pet in the store
  pet_id: Path<Uuid>,
) -> Result<Option<Json<Pet>>, Error> {
  Ok(None)
}
