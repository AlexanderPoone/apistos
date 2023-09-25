//! A set of macro utilities to generate [OpenAPI v3.0.3](https://github.com/OAI/OpenAPI-Specification/blob/main/versions/3.0.3.md) documentation from Rust models.
//!
//! ⚠️ This crate is not indented to be used by itself. Please use [**netwopenapi**](https://crates.io/crates/netwopenapi) instead.

use crate::internal::schemas::Schemas;
use crate::internal::utils::extract_deprecated_from_attr;
use crate::internal::{extract_generics_params, gen_item_ast, gen_open_api_impl};
use crate::openapi_cookie_attr::parse_openapi_cookie_attrs;
use crate::openapi_error_attr::parse_openapi_error_attrs;
use crate::openapi_header_attr::parse_openapi_header_attrs;
use crate::openapi_security_attr::parse_openapi_security_attrs;
use crate::operation_attr::parse_openapi_operation_attrs;
use convert_case::{Case, Casing};
use darling::ast::NestedMeta;
use darling::Error;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error, OptionExt};
use quote::quote;
use syn::{DeriveInput, Ident, ItemFn};

mod internal;
mod openapi_cookie_attr;
mod openapi_error_attr;
mod openapi_header_attr;
mod openapi_security_attr;
mod operation_attr;

const OPENAPI_STRUCT_PREFIX: &str = "__openapi_";

/// Generate a custom OpenAPI type.
///
/// This `#[derive]` macro should be used in combinaison with [TypedSchema](trait.TypedSchema.html).
///
/// When deriving [ApiType], [ApiComponent] and [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) are automatically implemented and thus
/// should not be derived.
///
/// ```rust
/// use netwopenapi::{ApiType, InstanceType, TypedSchema};
///
/// #[derive(Debug, Clone, ApiType)]
/// pub struct Name(String);
///
/// impl TypedSchema for Name {
///   fn schema_type() -> InstanceType {
///     InstanceType::String
///   }
///
///   fn format() -> Option<String> {
///     None
///   }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(ApiType)]
pub fn derive_api_type(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs: _attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  let component_name = quote!(#ident).to_string();
  quote!(
    #[automatically_derived]
    impl #generics schemars::JsonSchema for #ident #ty_generics #where_clause {
       fn is_referenceable() -> bool {
        false
      }

      fn schema_name() -> String {
        #component_name.to_string()
      }

      fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> netwopenapi::Schema {
        let instance_type = <Self as TypedSchema>::schema_type();
        netwopenapi::Schema::Object(netwopenapi::SchemaObject {
          instance_type: Some(netwopenapi::SingleOrVec::Single(Box::new(instance_type))),
          format: <Self as TypedSchema>::format(),
          ..Default::default()
        })
      }
    }

    #[automatically_derived]
    impl #generics netwopenapi::ApiComponent for #ident #ty_generics #where_clause {
      fn child_schemas() -> Vec<(String, netwopenapi::reference_or::ReferenceOr<netwopenapi::Schema>)> {
        vec![]
      }

      fn schema() -> Option<(String, netwopenapi::reference_or::ReferenceOr<netwopenapi::Schema>)> {
        Some((
          #component_name.to_string(),
          netwopenapi::reference_or::ReferenceOr::Object(netwopenapi::Schema::Object(netwopenapi::SchemaObject {
            instance_type: Some(netwopenapi::SingleOrVec::Single(Box::new(<#ident #ty_generics>::schema_type()))),
            format: <#ident #ty_generics>::format(),
            ..Default::default()
          }))
        ))
      }
    }
  )
  .into()
}

/// Generate a reusable OpenAPI schema.
///
/// This `#[derive]` macro should be used in combinaison with [api_operation](attr.api_operation.html).
///
/// This macro require your type to derive [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html).
///
/// ```rust
/// use netwopenapi::ApiComponent;
/// use schemars::JsonSchema;
/// use garde::Validate;
///
/// #[derive(Debug, Clone, JsonSchema, ApiComponent, Validate)]
/// pub(crate) struct QueryTag {
///   #[garde(length(min = 2))]
///   #[schemars(length(min = 2))]
///   pub(crate) tags: Vec<String>,
/// }
/// ```
///
/// Because this macro require [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html), all attributes supported by [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) are forward to
/// this implementation.
#[proc_macro_error]
#[proc_macro_derive(ApiComponent)]
pub fn derive_api_component(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs: _attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  let schema_impl = Schemas;
  quote!(
    #[automatically_derived]
    impl #generics netwopenapi::ApiComponent for #ident #ty_generics #where_clause {
      #schema_impl
    }
  )
  .into()
}

/// Generate a reusable OpenAPI security scheme.
///
/// This `#[derive]` macro should be used in combinaison with [api_operation](attr.api_operation.html).
/// The macro require one and only one `openapi_security`.
///
/// ```rust
/// use netwopenapi::ApiSecurity;
///
/// #[derive(ApiSecurity)]
/// #[openapi_security(scheme(security_type(api_key(name = "api_key", api_key_in = "header"))))]
/// pub struct ApiKey;
/// ```
///
/// # `#[openapi_security(...)]` options:
/// - `name = "..."` an optional name for your security definition. If not provided, the struct ident will be used.
/// - `scheme(...)` a **required** parameter with:
///   - `description = "..."` an optional description
///   - `security_type(...)` a **required** parameter with one of
///     - `oauth2(flows(...))` with
///       - `implicit(...)` with `authorization_url = "..."` a **required** parameter, `refresh_url = "..."` an optional parameter and `scopes(scope = "...", description = "...")` a list of scopes
///       - `password(...)` with `token_url = "..."` a **required** parameter, `refresh_url = "..."` an optional parameter and `scopes(scope = "...", description = "...")` a list of scopes
///       - `client_credentials(...)` with `token_url = "..."` a **required** parameter, `refresh_url = "..."` an optional parameter and `scopes(scope = "...", description = "...")` a list of scopes
///       - `authorization_code(...)` with `token_url = "..."` a **required** parameter, `refresh_url = "..."` an optional parameter and `scopes(scope = "...", description = "...")` a list of scopes
///     - `api_key(...)` with
///       - `name = "..."` a **required** parameter
///       - `api_key_in = "..."` a **required** parameter being one of `query`, `header` or `cookie`
///     - `http(...)` with
///       - `scheme = "..."` a **required** parameter
///       - `bearer_format = "..."` a **required** parameter
///     - `open_id_connect(open_id_connect_url = "...")`
///
/// # Examples:
///
/// ## **oauth2**
/// ```rust
/// use netwopenapi::ApiSecurity;
///
/// #[derive(ApiSecurity)]
/// #[openapi_security(scheme(security_type(oauth2(flows(implicit(
///   authorization_url = "https://authorize.com",
///   refresh_url = "https://refresh.com",
///   scopes(scope = "all:read", description = "Read all the things"),
///   scopes(scope = "all:write", description = "Write all the things")
/// ))))))]
/// pub struct ApiKey;
/// ```
///
/// ## **api_key**
/// ```rust
/// use netwopenapi::ApiSecurity;
///
/// #[derive(ApiSecurity)]
/// #[openapi_security(scheme(security_type(api_key(name = "api_key", api_key_in = "header"))))]
/// pub struct ApiKey;
/// ```
///
/// ## **http**
/// ```rust
/// use netwopenapi::ApiSecurity;
///
/// #[derive(ApiSecurity)]
/// #[openapi_security(scheme(security_type(http(scheme = "bearer", bearer_format = "JWT"))))]
/// pub struct ApiKey;
/// ```
///
/// ## **open_id_connect**
/// ```rust
/// use netwopenapi::ApiSecurity;
///
/// #[derive(ApiSecurity)]
/// #[openapi_security(scheme(security_type(open_id_connect(open_id_connect_url = "https://connect.com"))))]
/// pub struct ApiKey;
/// ```
#[proc_macro_error]
#[proc_macro_derive(ApiSecurity, attributes(openapi_security))]
pub fn derive_api_security(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let security_name: String = ident.to_string().to_case(Case::Snake);
  let openapi_security_attributes = parse_openapi_security_attrs(&attrs, security_name).expect_or_abort(
    "expected #[openapi_security(...)] attribute to be present when used with ApiSecurity derive trait",
  );
  let security_name = &openapi_security_attributes.name;

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  quote!(
    #[automatically_derived]
    impl #generics netwopenapi::ApiComponent for #ident #ty_generics #where_clause {
      fn child_schemas() -> Vec<(String, netwopenapi::reference_or::ReferenceOr<netwopenapi::Schema>)> {
        vec![]
      }

      fn schema() -> Option<(String, netwopenapi::reference_or::ReferenceOr<netwopenapi::Schema>)> {
        None
      }

      fn securities() -> std::collections::BTreeMap<String, netwopenapi::security::SecurityScheme> {
        #openapi_security_attributes
      }

      fn security_requirement_name() -> Option<String> {
        Some(#security_name.to_string())
      }
    }
  )
  .into()
}

/// Generate a reusable OpenAPI header schema.
///
/// This `#[derive]` macro should be used in combinaison with [api_operation](attr.api_operation.html).
/// The macro require one and only one `openapi_header`.
///
/// This macro require your type to derive [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html).
///
/// ```rust
/// use netwopenapi::ApiHeader;
/// use schemars::JsonSchema;
///
/// #[derive(Debug, Clone, JsonSchema, ApiHeader)]
/// #[openapi_header(
///   name = "X-Organization-Slug",
///   description = "Organization of the current caller",
///   required = true
/// )]
/// pub struct OrganizationSlug(String);
/// ```
///
/// # `#[openapi_header(...)]` options:
/// - `name = "..."` a **required** parameter with the header name
/// - `description = "..."` an optional description for the header
/// - `required = false` an optional parameter, default value is false
/// - `deprecated = false` an optional parameter, default value is false
///
/// Because this macro require [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html), all attributes supported by [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) are forward to
/// this implementation.
#[proc_macro_error]
#[proc_macro_derive(ApiHeader, attributes(openapi_header))]
pub fn derive_api_header(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let deprecated = extract_deprecated_from_attr(&attrs);

  let openapi_header_attributes = parse_openapi_header_attrs(&attrs, deprecated)
    .expect_or_abort("expected #[openapi_header(...)] attribute to be present when used with ApiHeader derive trait");

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  let schema_impl = Schemas;
  quote!(
    #[automatically_derived]
    impl #generics netwopenapi::ApiComponent for #ident #ty_generics #where_clause {
      #schema_impl
    }

    #[automatically_derived]
    impl #generics netwopenapi::ApiHeader for #ident #ty_generics #where_clause {
      #openapi_header_attributes
    }
  )
  .into()
}

/// Generate a reusable OpenAPI parameter schema in cookie.
///
/// This `#[derive]` macro should be used in combinaison with [api_operation](attr.api_operation.html).
/// The macro require one and only one `openapi_cookie`.
///
/// This macro require your type to derive [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html).
///
/// ```rust
/// use netwopenapi::ApiCookie;
/// use schemars::JsonSchema;
///
/// #[derive(Debug, Clone, JsonSchema, ApiCookie)]
/// #[openapi_cookie(
///   name = "X-Organization-Slug",
///   description = "Organization of the current caller",
///   required = true
/// )]
/// pub struct OrganizationSlugCookie(String);
/// ```
///
/// # `#[openapi_cookie(...)]` options:
/// - `name = "..."` a **required** parameter with the header name
/// - `description = "..."` an optional description for the header
/// - `required = false` an optional parameter, default value is false
/// - `deprecated = false` an optional parameter, default value is false
///
/// Because this macro require [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html), all attributes supported by [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) are forward to
/// this implementation.
#[proc_macro_error]
#[proc_macro_derive(ApiCookie, attributes(openapi_cookie))]
pub fn derive_api_cookie(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let deprecated = extract_deprecated_from_attr(&attrs);

  let openapi_cookie_attributes = parse_openapi_cookie_attrs(&attrs, deprecated)
    .expect_or_abort("expected #[openapi_cookie(...)] attribute to be present when used with ApiCookie derive trait");

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  quote!(
    #[automatically_derived]
    impl #generics netwopenapi::ApiComponent for #ident #ty_generics #where_clause {
      #openapi_cookie_attributes
    }
  )
  .into()
}

/// Generate a reusable OpenAPI error schema.
///
/// This `#[derive]` macro should be used in combinaison with [api_operation](attr.api_operation.html).
/// The macro accept one and only one `openapi_error`.
///
/// ```rust
/// use netwopenapi::ApiErrorComponent;
///
/// #[derive(Clone, ApiErrorComponent)]
/// #[openapi_error(
///   status(code = 403),
///   status(code = 404),
///   status(code = 405, description = "Invalid input"),
///   status(code = 409)
/// )]
/// pub enum ErrorResponse {
///   MethodNotAllowed(String),
///   NotFound(String),
///   Conflict(String),
///   Unauthorized(String),
/// }
/// ```
///
/// # `#[openapi_error(...)]` options:
/// - `status(...)` a list of possible error status with
///   - `code = 000` a **required** http status code
///   - `description = "..."` an optional description, default is the canonical reason of the given status code
///   - `with_schema = false` an optional parameter to indicate whether or not a schema should be included for this error definition in which case [JsonSchema](https://docs.rs/schemars/latest/schemars/trait.JsonSchema.html) should be derived
#[proc_macro_error]
#[proc_macro_derive(ApiErrorComponent, attributes(openapi_error))]
pub fn derive_api_error(input: TokenStream) -> TokenStream {
  let input = syn::parse_macro_input!(input as DeriveInput);
  let DeriveInput {
    attrs,
    ident,
    data: _data,
    generics,
    vis: _vis,
  } = input;

  let openapi_error_attributes = parse_openapi_error_attrs(&attrs).expect_or_abort(
    "expected #[openapi_error(...)] attribute to be present when used with ApiErrorComponent derive trait",
  );

  let (_, ty_generics, where_clause) = generics.split_for_impl();
  quote!(
    #[automatically_derived]
    impl #generics netwopenapi::ApiErrorComponent for #ident #ty_generics #where_clause {
      #openapi_error_attributes
    }
  )
  .into()
}

/// Operation attribute macro implementing [PathItemDefinition](path_item_definition/trait.PathItemDefinition.html) for the decorated handler function.
///
/// ```rust
/// use std::fmt::Display;
/// use actix_web::web::Json;
/// use actix_web::http::StatusCode;
/// use actix_web::ResponseError;
/// use core::fmt::Formatter;
/// use netwopenapi::actix::CreatedJson;
/// use netwopenapi::{api_operation, ApiComponent, ApiErrorComponent};
/// use schemars::JsonSchema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
/// pub struct Test {
///   pub test: String
/// }
///
/// #[derive(Serialize, Deserialize, Debug, Clone, ApiErrorComponent)]
/// #[openapi_error(
///   status(code = 405, description = "Invalid input"),
/// )]
/// pub enum ErrorResponse {
///   MethodNotAllowed(String),
/// }
///
/// impl Display for ErrorResponse {
///   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///     todo!()
///   }
/// }
///
/// impl ResponseError for ErrorResponse {
///   fn status_code(&self) -> StatusCode {
///     todo!()
///   }
/// }
///
/// #[api_operation(
///   tag = "pet",
///   summary = "Add a new pet to the store",
///   description = r###"Add a new pet to the store
///     Plop"###,
///   error_code = 405
/// )]
/// pub(crate) async fn test(
///   // Create a new pet in the store
///   body: Json<Test>,
/// ) -> Result<CreatedJson<Test>, ErrorResponse> {
///   Ok(CreatedJson(body.0))
/// }
/// ```
///
/// # `#[api_operation(...)]` options:
///   - `skip` a bool allowing to skip documentation for the decorated handler. No component
///  strictly associated to this operation will be document in the resulting openapi definition.
///   - `deprecated` a bool indicating the operation is deprecated. Deprecation can also be declared
///  with rust `#[deprecated]` decorator.
///   - `operation_id = "..."` an optional operation id for this operation. Default is the handler fn name.
///   - `summary = "..."` an optional summary
///   - `description = "..."` an optional description
///   - `tag = "..."` an optional list of tags associated to this operation (define tag multiple time to add to the list)
///   - `security_scope(...)` an optional list representing which security scopes apply for a given operation with
///       - `name = "..."` a mandatory name referencing one of the security definition
///       - `scope(...)` a list of scope applying to this operation
///   - `error_code = 00` an optional list of error code to document only theses
///
/// If `summary` or `description` are not provided, default value will be extracted from comments. The first line is used as summary while the rest will be part of the description.
///
/// For example:
/// ```rust
/// use actix_web::web::Json;
/// use std::fmt::Display;
/// use actix_web::http::StatusCode;
/// use actix_web::ResponseError;
/// use core::fmt::Formatter;
/// use netwopenapi::actix::CreatedJson;
/// use netwopenapi::{api_operation, ApiComponent, ApiErrorComponent};
/// use schemars::JsonSchema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
/// pub struct Test {
///   pub test: String
/// }
///
/// #[derive(Serialize, Deserialize, Debug, Clone, ApiErrorComponent)]
/// #[openapi_error(
///   status(code = 405, description = "Invalid input"),
/// )]
/// pub enum ErrorResponse {
///   MethodNotAllowed(String),
/// }
///
/// impl Display for ErrorResponse {
///   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///     todo!()
///   }
/// }
///
/// impl ResponseError for ErrorResponse {
///   fn status_code(&self) -> StatusCode {
///     todo!()
///   }
/// }
///
/// #[api_operation(
///   tag = "pet",
///   summary = "Add a new pet to the store",
///   description = r###"Add a new pet to the store
///     Plop"###,
/// )]
/// pub(crate) async fn test(
///   // Create a new pet in the store
///   body: Json<Test>,
/// ) -> Result<CreatedJson<Test>, ErrorResponse> {
///   Ok(CreatedJson(body.0))
/// }
/// ```
///
/// is equivalent to
/// ```rust
/// use std::fmt::Display;
/// use actix_web::web::Json;
/// use actix_web::http::StatusCode;
/// use actix_web::ResponseError;
/// use core::fmt::Formatter;
/// use netwopenapi::actix::CreatedJson;
/// use netwopenapi::{api_operation, ApiComponent, ApiErrorComponent};
/// use schemars::JsonSchema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, ApiComponent)]
/// pub struct Test {
///   pub test: String
/// }
///
/// #[derive(Serialize, Deserialize, Debug, Clone, ApiErrorComponent)]
/// #[openapi_error(
///   status(code = 405, description = "Invalid input"),
/// )]
/// pub enum ErrorResponse {
///   MethodNotAllowed(String),
/// }
///
/// impl Display for ErrorResponse {
///   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///     todo!()
///   }
/// }
///
/// impl ResponseError for ErrorResponse {
///   fn status_code(&self) -> StatusCode {
///     todo!()
///   }
/// }
///
/// /// Add a new pet to the store
/// /// Add a new pet to the store
/// /// Plop
/// #[api_operation(
///   tag = "pet",
/// )]
/// pub(crate) async fn test(
///   // Create a new pet in the store
///   body: Json<Test>,
/// ) -> Result<CreatedJson<Test>, ErrorResponse> {
///   Ok(CreatedJson(body.0))
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn api_operation(attr: TokenStream, item: TokenStream) -> TokenStream {
  let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
    Ok(v) => v,
    Err(e) => {
      return TokenStream::from(Error::from(e).write_errors());
    }
  };

  let operation_attribute = parse_openapi_operation_attrs(&attr_args);

  let default_span = proc_macro2::Span::call_site();
  let item_ast = match syn::parse::<ItemFn>(item) {
    Ok(v) => v,
    Err(e) => abort!(e.span(), format!("{e}")),
  };

  let s_name = format!("{OPENAPI_STRUCT_PREFIX}{}", item_ast.sig.ident);
  let openapi_struct = Ident::new(&s_name, default_span);

  let generics = &item_ast.sig.generics.clone();
  let mut generics_call = quote!();
  let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
  let openapi_struct_def = if !generics.params.is_empty() {
    let turbofish = ty_generics.as_turbofish();
    generics_call = quote!(#turbofish { p: std::marker::PhantomData });
    let generics_params = extract_generics_params(&item_ast);
    quote!(struct #openapi_struct #ty_generics { p: std::marker::PhantomData<(#generics_params)> } )
  } else {
    quote!(struct #openapi_struct;)
  };

  let (responder_wrapper, generated_item_ast) =
    gen_item_ast(default_span, item_ast, &openapi_struct, &ty_generics, &generics_call);
  let generated_item_fn = match syn::parse::<ItemFn>(generated_item_ast.clone().into()) {
    Ok(v) => v,
    Err(e) => abort!(e.span(), format!("{e}")),
  };
  let open_api_def = gen_open_api_impl(
    &generated_item_fn,
    operation_attribute,
    &openapi_struct,
    &openapi_struct_def,
    &impl_generics,
    &ty_generics,
    where_clause,
    &responder_wrapper,
  );

  quote!(
    #open_api_def

    #generated_item_ast
  )
  .into()
}
