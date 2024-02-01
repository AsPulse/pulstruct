use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
  parse::{ParseStream, Parser},
  punctuated::Punctuated,
  token::{self, Comma},
  Error, Fields, FnArg, ImplItem, ItemEnum, ItemImpl, Result, Variant,
};

pub fn pulstruct_api_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
  let name = attr.to_string();
  Parser::parse2(|i: ParseStream| pulstruct_api_parse(name, i), item)
    .unwrap_or_else(Error::into_compile_error)
}

fn pulstruct_api_parse(name: String, input: ParseStream) -> Result<TokenStream> {
  let item = input.parse::<ItemImpl>()?;

  let mut enum_items = Punctuated::<Variant, Comma>::new();
  item
    .items
    .iter()
    .filter_map(|i| {
      let ImplItem::Fn(function) = i else {
        // Skip non-function items
        return None;
      };
      let Some(FnArg::Receiver(_)) = function.sig.inputs.first() else {
        // Skip static functions
        return None;
      };
      let args = function
        .sig
        .inputs
        .iter()
        .skip(1)
        .map(|i| syn::Field {
          attrs: vec![],
          vis: syn::Visibility::Inherited,
          ident: None,
          colon_token: None,
          mutability: syn::FieldMutability::None,
          ty: if let FnArg::Typed(pat) = i {
            *pat.ty.clone()
          } else {
            panic!("Expected typed argument");
          },
        })
        .collect();
      Some(Variant {
        attrs: vec![],
        ident: function.sig.ident.clone(),
        fields: Fields::Unnamed(syn::FieldsUnnamed {
          paren_token: token::Paren::default(),
          unnamed: args,
        }),
        discriminant: None,
      })
    })
    .for_each(|i| {
      enum_items.push(i);
    });

  enum_items.push_punct(Comma::default());
  let enum_name = Ident::new(
    format!("__pulstruct_signature_{}", name).as_str(),
    Span::call_site(),
  );

  let enum_body = ItemEnum {
    attrs: vec![],
    vis: syn::Visibility::Inherited,
    enum_token: Default::default(),
    ident: enum_name.clone(),
    generics: Default::default(),
    brace_token: Default::default(),
    variants: enum_items,
  };

  Ok(quote! {
    #item
    #enum_body
  })
}

#[cfg(test)]
mod test {
  use pretty_assertions::assert_eq;
  use quote::quote;

  use crate::api::pulstruct_api_impl;

  #[test]
  fn convert_signatures_to_enum() {
    assert_eq!(
      pulstruct_api_impl(
        quote!(TestApi),
        quote! {
          impl Test {
            pub fn not_expose_static(a: String, b: String) -> bool {
              a == b
            }
            pub fn procedure_a(&self, a: String, b: Vec<u8>) -> bool {
              a == b
            }
            pub fn procedure_b(&self, a: some::path::UserStruct) -> bool {
              a == b
            }
          }
        }
      )
      .to_string(),
      quote! {
        impl Test {
          pub fn not_expose_static(a: String, b: String) -> bool {
            a == b
          }
          pub fn procedure_a(&self, a: String, b: Vec<u8>) -> bool {
            a == b
          }
          pub fn procedure_b(&self, a: some::path::UserStruct) -> bool {
            a == b
          }
        }
        enum __pulstruct_signature_TestApi {
          procedure_a(String, Vec<u8>),
          procedure_b(some::path::UserStruct),
        }
      }
      .to_string()
    )
  }

  #[test]
  fn should_fail_with_not_impl() {
    assert_eq!(
      pulstruct_api_impl(
        quote! {},
        quote! {
          struct Test {
            a: String,
          }
        }
      )
      .to_string(),
      ":: core :: compile_error ! { \"expected `impl`\" }"
    );
  }
}
