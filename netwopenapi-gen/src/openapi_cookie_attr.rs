use crate::internal::schemas::Schemas;
use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::Attribute;

pub(crate) fn parse_openapi_cookie_attrs(
  attrs: &[Attribute],
  deprecated: Option<bool>,
) -> Option<OpenapiCookieAttributeExtended> {
  let cookie_attribute = attrs
    .iter()
    .filter(|attribute| attribute.path().is_ident("openapi_cookie"))
    .map(|attribute| OpenapiCookieAttribute::from_meta(&attribute.meta))
    .collect::<darling::Result<Vec<OpenapiCookieAttribute>>>();

  match cookie_attribute {
    Ok(cookie_attributes) if cookie_attributes.len() > 1 => {
      abort!(Span::call_site(), "Expected only one #[openapi_cookie] attribute")
    }
    Ok(cookie_attributes) => {
      let cookie_attribute = cookie_attributes.first().cloned();
      cookie_attribute.map(|attr| OpenapiCookieAttributeExtended {
        name: attr.name,
        description: attr.description,
        required: attr.required,
        deprecated: attr.deprecated.or(deprecated),
      })
    }
    Err(e) => abort!(e.span(), "Unable to parse #[openapi_cookie] attribute: {:?}", e),
  }
}

#[derive(FromMeta, Clone)]
struct OpenapiCookieAttribute {
  pub name: String,
  pub description: Option<String>,
  pub required: Option<bool>,
  pub deprecated: Option<bool>,
}

#[derive(Clone)]
pub(crate) struct OpenapiCookieAttributeExtended {
  pub name: String,
  pub description: Option<String>,
  pub required: Option<bool>,
  pub deprecated: Option<bool>,
}

impl ToTokens for OpenapiCookieAttributeExtended {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let name = self.name.as_str();
    let description = match &self.description {
      None => quote!(None),
      Some(desc) => quote!(Some(#desc.to_string())),
    };
    let required = if self.required.unwrap_or_default() {
      quote!(utoipa::openapi::Required::True)
    } else {
      quote!(utoipa::openapi::Required::False)
    };
    let deprecated = if self.deprecated.unwrap_or_default() {
      quote!(utoipa::openapi::Deprecated::True)
    } else {
      quote!(utoipa::openapi::Deprecated::False)
    };

    let schema_impl = Schemas;
    tokens.extend(quote! {
      #schema_impl

      fn request_body() -> Option<utoipa::openapi::request_body::RequestBody> {
        None
      }

      fn parameters() -> Vec<utoipa::openapi::path::Parameter> {
        vec![utoipa::openapi::path::ParameterBuilder::new()
          .parameter_in(utoipa::openapi::path::ParameterIn::Cookie)
          .name(#name.to_string())
          .description(#description)
          .required(#required)
          .deprecated(Some(#deprecated))
          .schema(<Self as ApiComponent>::schema().map(|(_, schema)| schema).or_else(|| Self::raw_schema()))
          .build()]
      }
    })
  }
}
