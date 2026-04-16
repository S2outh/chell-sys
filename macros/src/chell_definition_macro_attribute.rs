use std::array::from_fn;
use std::iter::{once, zip};

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Item, Meta, Token, punctuated::Punctuated};
use syn::{MetaNameValue, Type};

const CHELL_VALUE_MACRO_NAME: &str = "chv";
const CHELL_MODULE_MACRO_NAME: &str = "chm";

struct TmValueMacroInput {
    pub ty: Type,
    pub metas: Punctuated<MetaNameValue, Token![,]>,
}

impl Parse for TmValueMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse first argument as a Type
        let ty: Type = input.parse()?;

        // If there's nothing else, return early
        if input.is_empty() {
            return Ok(Self {
                ty,
                metas: Punctuated::new(),
            });
        }

        // Expect comma after type
        input.parse::<Token![,]>()?;

        // Parse remaining key-value pairs
        let metas = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        Ok(Self { ty, metas })
    }
}

#[derive(Default, serde::Serialize)]
struct DefinitionDocumentation {
    base_address: String,
    id: u16,
    type_name: String,
    description: String,
    sub_addresses: Vec<String>,
}

#[derive(Default, serde::Serialize)]
struct FullDocumentation {
    base_address: String,
    definitions: Vec<DefinitionDocumentation>,
}

fn generate_struct(
    address: &Vec<syn::Ident>,
    id: &mut u16,
    docs: &mut Vec<DefinitionDocumentation>,
    v: &syn::ItemStruct,
) -> [TokenStream; 4] {
    // Parse "tmv" attribute
    let args: TmValueMacroInput = v
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(CHELL_VALUE_MACRO_NAME))
        .expect(&format!(
            "Struct {} has no {} attribute",
            &v.ident, CHELL_VALUE_MACRO_NAME
        ))
        .parse_args()
        .expect(&format!(
            "Could not parse {} attribute parameters",
            CHELL_VALUE_MACRO_NAME
        ));

    let tmty: Type = args.ty;

    let (address_endings, funcs): (Vec<_>, Vec<_>) =
        args.metas.into_iter().map(|v| (v.path, v.value)).unzip();

    // this definitions name
    let def = &v.ident;
    // Parse rust address of the struct inside the telemetry module tree
    let def_addr: TokenStream = address
        .iter()
        .skip(1)
        .chain(once(def))
        .map(|i| i.to_token_stream())
        .intersperse(quote!(::))
        .collect();
    // Parse type of the ChellValue the struct references
    // Increment id
    let tm_id = *id;
    *id += 1;
    // calculate string address based on module tree
    let str_base_addr: String = address
        .iter()
        .map(|i| i.to_string())
        .intersperse(String::from("."))
        .collect();
    // Parse address
    let address = format!("{}.{}", str_base_addr, def.to_string().to_snake_case());
    // generated documentation
    let mut doc = DefinitionDocumentation::default();
    doc.base_address = address.clone();
    doc.id = tm_id;
    if let Some(description) = v
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("doc"))
        .map(|v| {
            let Meta::NameValue(v) = &v.meta else {
                panic!()
            };
            v.value.to_token_stream().to_string()
        })
    {
        doc.description = description;
    }

    for addr in address_endings.iter() {
        let full_addr = format!("{}.{}", address, &addr.to_token_stream().to_string());
        doc.sub_addresses.push(full_addr);
    }
    doc.type_name = tmty.to_token_stream().to_string();

    let str_doc = serde_json::to_string(&doc).unwrap_or(String::new());
    docs.push(doc);

    // Serializer func
    let serializer_func = if cfg!(feature = "ground") {
        quote! {
            impl SerializableChellValue<#def> for #tmty {
                fn serialize_ground(self,
                    _def: &#def,
                    timestamp: &dyn Serialize,
                    serializer: &dyn Fn(&dyn Serialize) -> Result<Vec<u8>, Error>
                ) -> Result<Vec<(&'static str, Vec<u8>)>, Error> {
                    let mut serialized_pairs = Vec::new();
                    #({
                        let converted_value = (#funcs)(&self);
                        let nats_value = GroundTelemetry::new(timestamp, &converted_value);
                        let bytes = serializer(&nats_value)?;
                        serialized_pairs.push((concat!(#address, ".", stringify!(#address_endings)), bytes));
                    })*

                    let raw_nats_value = GroundTelemetry::new(timestamp, &self);
                    let raw_bytes = serializer(&raw_nats_value)?;
                    serialized_pairs.push((#address, raw_bytes));

                    Ok(serialized_pairs)
                }
            }
        }
    } else {
        quote! {}
    };
    let reserializer_func = if cfg!(feature = "ground") {
        quote! {
            fn reserialize(&self,
                bytes: &[u8],
                timestamp: &dyn Serialize,
                serializer: &dyn Fn(&dyn Serialize) -> Result<Vec<u8>, Error>
            ) -> Result<Vec<(&'static str, Vec<u8>)>, ReserializeError> {
                let (_, value): (_, #tmty) = <#tmty>::read(bytes)
                    .map_err(|e| ReserializeError::ChellValueError(e))?;
                let serialized_pairs = value.serialize_ground(&self, timestamp, serializer)
                    .map_err(|e| ReserializeError::SerdeError(e))?;

                Ok(serialized_pairs)
            }

        }
    } else {
        quote! {}
    };
    [
        quote! {
            #[doc = #str_doc]
            pub struct #def;
            impl InternalChellDefinition for #def {
                type ChellValueType = #tmty;
                const ID: u16 = #tm_id;
            }
            impl ChellDefinition for #def {
                fn id(&self) -> u16 { Self::ID }
                fn address(&self) -> &str { #address }
                fn as_any(&self) -> &dyn Any { self }
                #reserializer_func
            }
            impl #def {
                fn equals(&self, other: &dyn ChellDefinition) -> bool {
                    self.type_id() == other.type_id()
                }
            }
            #serializer_func
        },
        quote! {
            #tm_id => Ok(&#def_addr),
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
    docs: &mut Vec<DefinitionDocumentation>,
    v: &syn::ItemMod,
) -> [TokenStream; 4] {
    // Parse "tmm" attribute
    if let Some(module_id) = v
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(CHELL_MODULE_MACRO_NAME))
        .map(|v| {
            v.parse_args_with(Punctuated::<Meta, Token![,]>::parse_separated_nonempty)
                .expect(&format!(
                    "Could not parse {} attribute parameters",
                    CHELL_MODULE_MACRO_NAME
                ))
                .iter()
                .filter_map(|m| m.require_name_value().ok())
                .filter(|m| m.path.get_ident().filter(|p| *p == "id").is_some())
                .map(|m| {
                    if let syn::Expr::Lit(value) = &m.value {
                        value
                    } else {
                        panic!("unexpected attribute value type")
                    }
                })
                .map(|m| {
                    if let syn::Lit::Int(value) = &m.lit {
                        value
                    } else {
                        panic!("unexpected attribute value type")
                    }
                })
                .next()
                .map(|lit| lit.base10_parse().unwrap())
        })
        .flatten()
    {
        if *id > module_id {
            panic!("like schedules, ids should only move in one direction");
        }
        *id = module_id;
    }
    let start_id = *id;

    let module_name = v.ident.clone();
    let mut address = address.clone();
    address.push(module_name.clone());
    let [module_content, id_getters, address_getters, byte_lengths] = generate_tree(
        address,
        id,
        docs,
        &v.content.as_ref().expect("module sould not be empty").1,
    );

    [
        quote! {
            pub mod #module_name {
                use super::*;
                pub const fn id_range() -> (u16, u16) {
                    (#start_id, #id)
                }
                pub const MAX_BYTE_SIZE: usize = {
                    let SIZES = [#byte_lengths];
                    let mut max = 0;
                    let mut i = 0;
                    while i < SIZES.len() {
                        if SIZES[i] > max {
                            max = SIZES[i];
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

fn generate_tree(
    address: Vec<syn::Ident>,
    id: &mut u16,
    docs: &mut Vec<DefinitionDocumentation>,
    items: &Vec<Item>,
) -> [TokenStream; 4] {
    items
        .iter()
        .map(|v| match v {
            syn::Item::Struct(v) => generate_struct(&address, id, docs, v),
            syn::Item::Mod(v) => generate_module_recursive(&address, id, docs, v),
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

    let mut doc = FullDocumentation::default();
    doc.base_address = root_mod_ident.to_string();

    let [module_content, id_getters, address_getters, byte_lengths] = generate_tree(
        vec![root_mod_ident.clone()],
        id_ref,
        &mut doc.definitions,
        &root_mod_content.1,
    );

    let str_doc = serde_json::to_string(&doc).unwrap_or(String::new());

    let serializer_imports = if cfg!(feature = "ground") {
        quote! {
            use alloc::vec::Vec;
            use erased_serde::{Serialize, Error};
        }
    } else {
        quote! {}
    };

    quote! {
        pub mod #root_mod_ident {
            pub const __TOOLING_METADATA: &str = #str_doc;
            use #chell_address::{*, _internal::*};
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
                let SIZES = [#byte_lengths];
                let mut max = 0;
                let mut i = 0;
                while i < SIZES.len() {
                    if SIZES[i] > max {
                        max = SIZES[i];
                    }
                    i += 1;
                }
                max
            };
            #module_content
        }
    }
}
