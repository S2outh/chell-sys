#![feature(iter_intersperse)]
mod beacon_macro;
mod chell_definition_macro_attribute;
mod chell_value_macro_derive;
mod macro_utils;
use std::panic;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Meta, MetaNameValue, Token, parse_macro_input, parse2, punctuated::Punctuated};

#[proc_macro_derive(ChellValue)]
pub fn chell_value(item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();

    // Build the ChellValue and DynChellValue trait implementations
    chell_value_macro_derive::impl_macro(ast).into()
}

#[proc_macro]
pub fn beacon(input: TokenStream) -> TokenStream {
    let args =
        syn::parse_macro_input!(input with Punctuated<Meta, Token![,]>::parse_separated_nonempty);

    // Build the beacon definition and implementation
    beacon_macro::impl_macro(args).into()
}

#[proc_macro_attribute]
pub fn chell_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse(item).unwrap();
    let name_value_pairs = parse_macro_input!(attr with Punctuated<MetaNameValue, Token![,]>::parse_separated_nonempty);
    if name_value_pairs
        .get(0)
        .expect("missing id")
        .path
        .get_ident()
        .expect("invalid attribute")
        != "id"
    {
        panic!("first attr in chell definition must be id");
    }
    let syn::Expr::Lit(id_expr) = name_value_pairs[0].value.clone() else {
        panic!("wrong macro attributes")
    };
    let syn::Lit::Int(id_lit) = id_expr.lit else {
        panic!("wrong macro attributes")
    };
    let id = id_lit
        .base10_parse::<u16>()
        .expect("macro input should be an u16");

    let chell_address = if let Some(address_name_value) = name_value_pairs.get(1) {
        if address_name_value
            .path
            .get_ident()
            .expect("invalid attribute")
            != "address"
        {
            panic!("second attr should be chell base address or nothing");
        }
        let syn::Expr::Path(path_addr) = address_name_value.value.clone() else {
            panic!("wrong macro attributes");
        };
        path_addr.path
    } else {
        parse2(quote! { chell }).unwrap()
    };

    // Build the telemetry definition recursive module
    chell_definition_macro_attribute::impl_macro(ast, id, chell_address).into()
}
