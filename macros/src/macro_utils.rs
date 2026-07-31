use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Type;

pub fn parse_type_path(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(syn::TypePath { path, .. }) => path
            .segments
            .iter()
            .map(|s| {
                let ident = &s.ident;
                match &s.arguments {
                    syn::PathArguments::None => s.ident.to_token_stream(),
                    syn::PathArguments::AngleBracketed(args) => quote! {#ident::#args},
                    syn::PathArguments::Parenthesized(_) => {
                        panic!("Parenthesized types are unsupported")
                    }
                }
            })
            .collect(),
        Type::Array(a) => quote! {<#a>},
        _ => panic!("unsupported type"),
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub trait PartitionMapExt: Iterator + Sized {
    fn partition_map<L, R, BL, BR, F>(self, f: F) -> (BL, BR)
    where
        F: FnMut(&Self::Item) -> Either<L, R>,
        BL: Default + Extend<L>,
        BR: Default + Extend<R>;
}

impl<I: Iterator> PartitionMapExt for I {
    fn partition_map<L, R, BL, BR, F>(self, f: F) -> (BL, BR)
    where
        F: FnMut(&Self::Item) -> Either<L, R>,
        BL: Default + Extend<L>,
        BR: Default + Extend<R>,
    {
        #[inline]
        fn extend<'a, T, L, R, BL: Extend<L>, BR: Extend<R>>(
            mut f: impl FnMut(&T) -> Either<L, R> + 'a,
            left: &'a mut BL,
            right: &'a mut BR,
        ) -> impl FnMut((), T) + 'a {
            move |(), v| match f(&v) {
                Either::Left(x) => left.extend_one(x),
                Either::Right(x) => right.extend_one(x),
            }
        }
        let mut left = BL::default();
        let mut right = BR::default();

        self.fold((), extend(f, &mut left, &mut right));

        (left, right)
    }
}
