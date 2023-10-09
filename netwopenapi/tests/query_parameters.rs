#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use actix_web::web::Query;
use netwopenapi_core::ApiComponent;
use netwopenapi_gen::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[actix_web::test]
async fn query_parameters() {
  #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
  pub(crate) enum StatusQuery {
    Active,
    Inactive,
  }

  #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
  pub(crate) struct PaginationQuery {
    pub(crate) limit: u32,
    pub(crate) offset: Option<u32>,
  }

  #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, ApiComponent, JsonSchema)]
  pub(crate) struct StructQuery {
    pub(crate) test: Option<String>,
    pub(crate) status: Option<StatusQuery>,
    #[serde(flatten)]
    pub(crate) pagination: PaginationQuery,
  }

  let parameters = <Query<StructQuery> as ApiComponent>::parameters();
  assert_eq!(parameters.len(), 4);

  let test_parameter = parameters
    .iter()
    .find(|p| p.name == *"test")
    .expect("Unable to retrieve test parameter");
  assert_eq!(test_parameter.required, Some(false));

  let status_parameter = parameters
    .iter()
    .find(|p| p.name == *"status")
    .expect("Unable to retrieve status parameter");
  assert_eq!(status_parameter.required, Some(false));

  let limit_parameter = parameters
    .iter()
    .find(|p| p.name == *"limit")
    .expect("Unable to retrieve limit parameter");
  assert_eq!(limit_parameter.required, Some(true));

  let offset_parameter = parameters
    .iter()
    .find(|p| p.name == *"offset")
    .expect("Unable to retrieve offset parameter");
  assert_eq!(offset_parameter.required, Some(false));
}

// Imports bellow aim at making cargo-cranky happy. Those dependencies are necessary for integration-test.
use actix_service as _;
use indexmap as _;
use log as _;
use netwopenapi_models as _;
use once_cell as _;
use regex as _;
use serde_json as _;
