use std::array::from_fn;
use std::iter::{once, zip};

use heck::{ToSnakeCase, ToTitleCase};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, MetaNameValue, Type};
use syn::{Item, Token};

const TYPE_MACRO_IDENT: &str = "def";
const PARSER_MACRO_IDENT: &str = "parser";
const ID_MACRO_IDENT: &str = "id";
const COMMAND_MACRO_IDENT: &str = "command";
const DOC_MACRO_IDENT: &str = "doc";

struct DefMacroArgs {
    ty: Type,
    doctype: Option<String>,
}

impl Parse for DefMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let mut args = Self { ty, doctype: None };
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let kv_pair: MetaNameValue = input.parse()?;
            let key = kv_pair.path.get_ident().unwrap().to_string();
            let value = kv_pair.value;
            match key.as_str() {
                "doc" => {
                    let Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(doctype),
                        ..
                    }) = value
                    else {
                        return Err(syn::Error::new(input.span(), "Wrong expression"));
                    };
                    args.doctype = Some(doctype.value());
                }
                _ => return Err(syn::Error::new(input.span(), "Wrong expression")),
            }
        }
        Ok(args)
    }
}

struct ParserMacroArgs {
    name: Ident,
    ty: Type,
    expr: Expr,
    doctype: Option<String>,
}

impl Parse for ParserMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![->]>()?;
        let ty: Type = input.parse()?;
        let expr: Expr = input.parse()?;
        let mut args = Self {
            name,
            ty,
            expr,
            doctype: None,
        };
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let kv_pair: MetaNameValue = input.parse()?;
            let key = kv_pair.path.get_ident().unwrap().to_string();
            let value = kv_pair.value;
            match key.as_str() {
                "doc" => {
                    let Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(doctype),
                        ..
                    }) = value
                    else {
                        return Err(syn::Error::new(input.span(), "Wrong expression"));
                    };
                    args.doctype = Some(doctype.value());
                }
                _ => return Err(syn::Error::new(input.span(), "Wrong expression")),
            }
        }
        Ok(args)
    }
}

struct IdMacroArgs {
    range: u16,
}

impl Parse for IdMacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args: Self = Self { range: 0 };
        while !input.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            let kv_pair: MetaNameValue = input.parse()?;
            let key = kv_pair.path.get_ident().unwrap().to_string();
            let value = kv_pair.value;
            match key.as_str() {
                "range" => {
                    let Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(doctype),
                        ..
                    }) = value
                    else {
                        return Err(syn::Error::new(input.span(), "Wrong expression"));
                    };
                    args.range = doctype.base10_parse().unwrap();
                }
                _ => return Err(syn::Error::new(input.span(), "Wrong expression")),
            }
        }
        Ok(args)
    }
}

fn generate_struct(
    address: &Vec<syn::Ident>,
    id: &mut u16,
    v: &syn::ItemStruct,
) -> [TokenStream; 4] {
    // Parse attributes
    let DefMacroArgs {
        ty: def_type,
        doctype,
    } = v
        .attrs
        .iter()
        .find(|v| v.path().is_ident(TYPE_MACRO_IDENT))
        .expect("No type macro found")
        .parse_args()
        .expect("could not parse type macro");

    let parser_args: Vec<ParserMacroArgs> = v
        .attrs
        .iter()
        .filter(|v| v.path().is_ident(PARSER_MACRO_IDENT))
        .map(|v| v.parse_args().expect("could not parse parser macro"))
        .collect();

    let id_args: Option<IdMacroArgs> = v
        .attrs
        .iter()
        .find(|v| v.path().is_ident(ID_MACRO_IDENT))
        .map(|v| v.parse_args().expect("could not parse id macro"));

    let is_command = v
        .attrs
        .iter()
        .any(|v| v.path().is_ident(COMMAND_MACRO_IDENT));

    let docs = v.attrs.iter().find(|v| v.path().is_ident(DOC_MACRO_IDENT));

    // verification
    assert!(
        !is_command || parser_args.is_empty(),
        "commands don't have parsers"
    );

    // this definitions name
    let def = &v.ident;
    // the name of this definitions parser
    let def_parser = format_ident!("{}Parser", def);
    // Parse rust address of the struct inside the telemetry module tree
    let def_addr: TokenStream = address
        .iter()
        .skip(1)
        .chain(once(def))
        .map(|i| i.to_token_stream())
        .intersperse(quote!(::))
        .collect();

    // Parse type of the ChellValue the struct references
    let (def_id, def_id_match, increment) = match id_args {
        Some(IdMacroArgs { range }) => {
            let end = *id + range as u16;
            (
                quote! { CanID::Range(#id, #range) },
                quote! { #id..#end },
                range as u16,
            )
        }
        None => (quote! { CanID::Single(#id) }, quote! { #id }, 1),
    };

    // increment id
    *id += increment;
    // calculate string address based on module tree
    let str_base_addr: String = address
        .iter()
        .map(|i| i.to_string())
        .intersperse(String::from("."))
        .collect();
    // this definitions name
    // Parse address
    let address = format!("{}.{}", str_base_addr, def.to_string().to_snake_case());

    // Parse doc info
    let label = def.to_string().to_title_case();
    let parser_docs = parser_args
        .iter()
        .map(|ParserMacroArgs { name, doctype, .. }| {
            let label = format!("{} - {}", label, name);
            quote! {
                #[doc = "[[parserd]]"]
                #[doc = concat!("label = {}", #label)]
                #[doc = concat!("address = {}.{}", #address, stringify!(#name))]
                #[doc = concat!("type = {}", #doctype)]
            }
        });

    // Impl Chell Parsers
    let parser_funcs = parser_args
        .iter()
        .map(|ParserMacroArgs { name, ty, expr, .. }| {
            quote! {
                pub fn #name(&self) -> #ty {
                    (#expr)(&self.0)
                }
            }
        });
    let parser = if parser_funcs.len() > 0 {
        Some(quote! {
            pub struct #def_parser(#def_type);
            impl ParsableChellValue<#def> for #def_type {
                type Parser = #def_parser;
                fn parser(self, _def: #def) -> Self::Parser {
                    #def_parser(self)
                }
            }
            impl #def_parser {
                #(#parser_funcs)*
            }
        })
    } else {
        None
    };

    // Ground serialization funktionality
    let serializer_funcs = parser_args
        .iter()
        .map(|ParserMacroArgs { name, ty, expr, .. }| {
            quote! {{
                let converted_value: #ty = (#expr)(&self);
                let gt_value = GroundTelemetry::new(timestamp, &converted_value);
                let bytes = serializer(&gt_value)?;
                serialized_pairs.push((concat!(#address, ".", stringify!(#name)), bytes));
            }}
        });
    let serializer = if cfg!(feature = "ground") {
        Some(quote! {
            impl SerializableChellValue<#def> for #def_type {
                fn serialize_ground(self,
                    _def: #def,
                    timestamp: &dyn Serialize,
                    serializer: &dyn Fn(&dyn Serialize) -> Result<Vec<u8>, Error>
                ) -> Result<Vec<(&'static str, Vec<u8>)>, Error> {
                    let mut serialized_pairs = Vec::new();

                    #(#serializer_funcs)*

                    let raw_gt_value = GroundTelemetry::new(timestamp, &self);
                    let raw_bytes = serializer(&raw_gt_value)?;
                    serialized_pairs.push((#address, raw_bytes));

                    Ok(serialized_pairs)
                }
            }
        })
    } else {
        None
    };
    let reserializer_func = if cfg!(feature = "ground") {
        Some(quote! {
            fn reserialize(&self,
                bytes: &[u8],
                timestamp: &dyn Serialize,
                serializer: &dyn Fn(&dyn Serialize) -> Result<Vec<u8>, Error>
            ) -> Result<Vec<(&'static str, Vec<u8>)>, ReserializeError> {
                let (_, value): (_, #def_type) = <#def_type>::read(bytes)
                    .map_err(|e| ReserializeError::ChellValueError(e))?;
                let serialized_pairs = value.serialize_ground(Self, timestamp, serializer)
                    .map_err(|e| ReserializeError::SerdeError(e))?;

                Ok(serialized_pairs)
            }

        })
    } else {
        None
    };
    [
        quote! {
            #[doc = concat!("# {}", stringify!(#def))]
            #docs
            #[doc = ""]
            #[doc = "```tooling"]
            #[doc = concat!("label = {}", #label)]
            #[doc = concat!("address = {}", #address)]
            #[doc = concat!("type = {}", #doctype)]
            #[doc = concat!("command = {}", #is_command)]
            #[doc = ""]
            #(#parser_docs)*
            #[doc = "```"]
            pub struct #def;
            impl InternalChellDefinition for #def {
                type ChellValueType = #def_type;
            }
            impl ChellDefinition for #def {
                fn id(&self) -> CanID { #def_id }
                fn address(&self) -> &str { #address }
                fn as_any(&self) -> &dyn Any { self }
                #reserializer_func
            }
            impl #def {
                pub fn equals(&self, other: &dyn ChellDefinition) -> bool {
                    self.type_id() == other.type_id()
                }
                pub fn serialize(&self, value: &#def_type) -> Result<[u8; Self::MAX_BYTE_SIZE], ChellValueError> {
                    let mut mem = [0; _];
                    value.write(&mut mem)?;
                    Ok(mem)
                }
                pub fn deserialize(&self, bytes: &[u8]) -> Result<#def_type, ChellValueError> {
                    let (_, v) = <#def_type>::read(bytes)?;
                    Ok(v)
                }
            }
            #parser
            #serializer
        },
        quote! {
            #def_id_match => Ok(&#def_addr),
        },
        quote! {
            #address => Ok(&#def_addr),
        },
        quote! {
            #def::MAX_BYTE_SIZE,
        },
    ]
}

fn generate_module_recursive(
    address: &Vec<syn::Ident>,
    id: &mut u16,
    v: &syn::ItemMod,
) -> [TokenStream; 4] {
    let start_id = *id;

    let module_name = v.ident.clone();
    let mut address = address.clone();
    address.push(module_name.clone());
    let str_base_addr: String = address
        .iter()
        .map(|i| i.to_string())
        .intersperse(String::from("."))
        .collect();
    let [module_content, id_getters, address_getters, byte_lengths] = generate_tree(
        address,
        id,
        &v.content.as_ref().expect("module should not be empty").1,
    );

    [
        quote! {
            pub mod #module_name {
                use super::*;
                pub const fn id_range() -> (u16, u16) {
                    (#start_id, #id)
                }
                pub const fn base_address() -> &'static str {
                    #str_base_addr
                }
                pub const MAX_BYTE_SIZE: usize = {
                    let sizes = [#byte_lengths];
                    let mut max = 0;
                    let mut i = 0;
                    while i < sizes.len() {
                        if sizes[i] > max {
                            max = sizes[i];
                        }
                        i += 1;
                    }
                    max
                };
                #module_content
            }
        },
        id_getters,
        address_getters,
        quote! {
            #module_name::MAX_BYTE_SIZE,
        },
    ]
}

fn generate_tree(address: Vec<syn::Ident>, id: &mut u16, items: &Vec<Item>) -> [TokenStream; 4] {
    items
        .iter()
        .map(|v| match v {
            syn::Item::Struct(v) => generate_struct(&address, id, v),
            syn::Item::Mod(v) => generate_module_recursive(&address, id, v),
            _ => panic!("module should only contain other modules and structs"),
        })
        .fold(from_fn(|_| TokenStream::new()), |acc, src| {
            zip(acc, src)
                .map(|(mut acc, src)| {
                    acc.extend(src);
                    acc
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        })
}

pub fn impl_macro(ast: syn::Item, mut id: u16, chell_address: syn::Path) -> TokenStream {
    let syn::Item::Mod(telem_defnition) = ast else {
        panic!("chell defintion is not a module");
    };

    let root_mod_ident = telem_defnition.ident;
    let Some(root_mod_content) = telem_defnition.content else {
        panic!("module is empty");
    };
    let start_id = id;
    let id_ref = &mut id;

    let [module_content, id_getters, address_getters, byte_lengths] =
        generate_tree(vec![root_mod_ident.clone()], id_ref, &root_mod_content.1);

    let serializer_imports = if cfg!(feature = "ground") {
        Some(quote! {
            use alloc::vec::Vec;
            use erased_serde::{Serialize, Error};
        })
    } else {
        None
    };

    quote! {
        pub mod #root_mod_ident {
            use super::*;
            use #chell_address::{definition::*, value::*, _internal::*};
            use core::any::Any;
            #serializer_imports
            pub const fn from_id(id: u16) -> Result<&'static dyn ChellDefinition, NotFoundError> {
                match id {
                    #id_getters
                    _ => Err(NotFoundError)
                }
            }
            pub const fn from_address(address: &str) -> Result<&'static dyn ChellDefinition, NotFoundError> {
                match address {
                    #address_getters
                    _ => Err(NotFoundError)
                }
            }
            pub const fn id_range() -> (u16, u16) {
                (#start_id, #id_ref)
            }
            pub const MAX_BYTE_SIZE: usize = {
                let sizes = [#byte_lengths];
                let mut max = 0;
                let mut i = 0;
                while i < sizes.len() {
                    if sizes[i] > max {
                        max = sizes[i];
                    }
                    i += 1;
                }
                max
            };
            #module_content
        }
    }
}
