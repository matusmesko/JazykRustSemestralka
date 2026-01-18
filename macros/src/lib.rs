mod entity;
mod query;
mod controller;

use proc_macro::TokenStream;
use quote::{ToTokens};

#[proc_macro_attribute]
pub fn Entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    entity::entity_impl(attr, item)
}


#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    query::query_impl(attr, item)
}

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::controller_impl(attr, item)
}