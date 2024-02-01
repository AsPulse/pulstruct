use api::pulstruct_api_impl;

mod api;
extern crate proc_macro;

#[proc_macro_attribute]
pub fn pulstruct_api(
  attr: proc_macro::TokenStream,
  item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  pulstruct_api_impl(attr.into(), item.into()).into()
}
