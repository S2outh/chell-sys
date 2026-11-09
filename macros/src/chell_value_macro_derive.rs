use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Index};

fn impl_struct(tm_value_struct: syn::DataStruct) -> TokenStream {
    let field_readers = tm_value_struct.fields.iter().enumerate().map(|(i, f)| {
        let ident = f
            .ident
            .as_ref()
            .map(Ident::to_token_stream)
            .unwrap_or(Index::from(i).to_token_stream());
        let ty = &f.ty;
        quote! {
            #ident: {
                let (len, value) = <#ty>::read(&bytes[pos..])?;
                pos += len;
                value
            }
        }
    });
    let field_writers = tm_value_struct.fields.iter().enumerate().map(|(i, f)| {
        let ident = f
            .ident
            .as_ref()
            .map(Ident::to_token_stream)
            .unwrap_or(Index::from(i).to_token_stream());
        quote! {
            pos += self.#ident.write(&mut mem[pos..])?;
        }
    });
    let struct_types = tm_value_struct.fields.iter().map(|f| &f.ty);
    quote! {
        const MAX_BYTE_SIZE: usize = #(<#struct_types as ChellValue>::MAX_BYTE_SIZE)+*;
        fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
            let mut pos = 0;
            let value = Self {
                #(#field_readers),*
            };
            Ok((pos, value))
        }
        fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
            let mut pos = 0;
            #(#field_writers)*
            Ok(pos)
        }
    }
}

fn impl_enum(tm_value_enum: syn::DataEnum, attrs: Vec<syn::Attribute>) -> TokenStream {
    // parse tag type from #[repr]. isize is default.
    let tag_type: syn::Type = attrs
        .iter()
        .find(|attr| attr.path().is_ident("repr"))
        .map(|attr| attr.parse_args().unwrap())
        .unwrap_or(syn::parse_quote!(isize));

    let variant_sizes = tm_value_enum.variants.iter().map(|v| {
        let iter: Box<dyn Iterator<Item = _>> = match &v.fields {
            syn::Fields::Unit => Box::new(std::iter::empty()),
            syn::Fields::Unnamed(unnamed_fields) => Box::new(unnamed_fields.unnamed.iter()),
            syn::Fields::Named(named_fields) => Box::new(named_fields.named.iter()),
        };
        let sizes = iter
            .map(|f| &f.ty)
            .map(|ty| quote! { <#ty>::MAX_BYTE_SIZE });
        quote! {
            let variant_size = <#tag_type>::MAX_BYTE_SIZE #(+ #sizes)*;
            if variant_size > m {
                m = variant_size;
            }
        }
    });
    let variant_readers = tm_value_enum.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let index = syn::LitInt::new(
            &format!("{}{}", i, tag_type.to_token_stream()),
            proc_macro2::Span::call_site(),
        );
        match &v.fields {
            syn::Fields::Unit => {
                quote! {
                    #index => {
                        Self::#ident
                    }
                }
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                let field_parsers = unnamed_fields.unnamed.iter().map(|v| {
                    let ty = &v.ty;
                    quote! {{
                        let (len, value) = <#ty>::read(&bytes[pos..])?;
                        pos += len;
                        value
                    }}
                });
                quote! {
                    #index => {
                        Self::#ident(#(#field_parsers),*)
                    }
                }
            }
            syn::Fields::Named(named_fields) => {
                let field_parsers = named_fields.named.iter().map(|v| {
                    let ty = &v.ty;
                    let field_name = &v.ident;
                    quote! {#field_name: {
                        let (len, value) = <#ty>::read(&bytes[pos..])?;
                        pos += len;
                        value
                    }}
                });
                quote! {
                    #index => {
                        Self::#ident{#(#field_parsers),*}
                    }
                }
            }
        }
    });
    let variant_writers = tm_value_enum.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let index = syn::LitInt::new(
            &format!("{}{}", i, tag_type.to_token_stream()),
            proc_macro2::Span::call_site(),
        );
        match &v.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#ident => {
                        pos += #index.write(mem)?;
                    }
                }
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                let field_idents = (0..unnamed_fields.unnamed.len())
                    .map(|i| Ident::new(&format!("v{}", i), proc_macro2::Span::call_site()));
                let field_parsers = field_idents.clone().map(|ident| {
                    quote! {
                        pos += #ident.write(&mut mem[pos..])?;
                    }
                });
                quote! {
                    Self::#ident(#(#field_idents),*) => {
                        pos += #index.write(mem)?;
                        #(#field_parsers)*
                    }
                }
            }
            syn::Fields::Named(named_fields) => {
                let field_idents = named_fields.named.iter().map(|v| &v.ident);
                let field_parsers = field_idents.clone().map(|ident| {
                    quote! {
                        pos += #ident.write(&mut mem[pos..])?;
                    }
                });
                quote! {
                    Self::#ident{#(#field_idents),*} => {
                        pos += #index.write(mem)?;
                        #(#field_parsers)*
                    }
                }
            }
        }
    });
    quote! {
        const MAX_BYTE_SIZE: usize = {
            let mut m = 0;
            #(#variant_sizes)*
            m
        };
        fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
            let (mut pos, tag) = #tag_type::read(bytes)?;
            let value = match tag {
                #(#variant_readers)*
                _ => return Err(ChellValueError::BadEnumVariant)
            };
            Ok((pos, value))
        }
        fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
            let mut pos = 0;
            match self {
                #(#variant_writers)*
            }
            Ok(pos)
        }
    }
}

pub fn impl_macro(ast: syn::DeriveInput) -> TokenStream {
    let type_name = ast.ident.clone();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let body = match ast.data {
        syn::Data::Struct(tm_value_struct) => impl_struct(tm_value_struct),
        syn::Data::Enum(tm_value_enum) => impl_enum(tm_value_enum, ast.attrs),
        syn::Data::Union(_) => unimplemented!("unions are not supported as chell values"),
    };

    quote! {
        impl #impl_generics ChellValue for #type_name #ty_generics #where_clause {
            #body
        }
    }
}
