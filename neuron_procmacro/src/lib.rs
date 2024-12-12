extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::__private::ext::RepToTokensExt;
use quote::quote;
use syn::parse::Parser;
use syn::{parse_macro_input, ItemImpl, ItemTrait, Path, PathSegment, TraitBound, TraitBoundModifier, TypeParamBound};

#[proc_macro_attribute]
pub fn sealed(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemTrait);


    let args_parsed = syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated
        .parse(args)
        .unwrap();

    let ident = input.ident.clone();

    let seal_ident = Ident::new(&format!("__{}Seal", ident), Span::call_site());

    let mut seal_impls = quote! {};
    for type_path in args_parsed {
        seal_impls.extend(quote! {
            impl #seal_ident for #type_path {}
        })
    }

    let seal_def = quote! {
        mod __seal {
            use super::*;

            pub(super) trait #seal_ident {}

            #seal_impls
        }
    };

    let mut trait_path = Path::from(Ident::new("__seal", Span::call_site()));
    trait_path.segments.push(PathSegment::from(seal_ident));

    input.supertraits.push(TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: trait_path,
    }));

    let tokens = quote! {
        #input

        #seal_def
    };

    tokens.into()
}



// use this like you are making an impl block for the target type, it'll rework that correctly.
#[proc_macro_attribute]
pub fn extend_type(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_parsed = syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated
        .parse(args)
        .unwrap();

    if args_parsed.len() != 1 { panic!("Wrong number of arguments!") }

    let input = parse_macro_input!(input as ItemImpl);

    if input.trait_.is_some() {
        panic!("Cannot extend type with a trait impl block")
    }

    let ty = input.self_ty.clone();



    let seal_ident = Ident::new(&format!("__{}_Ext{}_Seal", ), Span::call_site());

    let mut seal_impls = quote! {};
    for type_path in args_parsed {
        seal_impls.extend(quote! {
            impl #seal_ident for #type_path {}
        })
    }

    let seal_def = quote! {
        mod __seal {
            use super::*;

            pub(super) trait #seal_ident {}

            #seal_impls
        }
    };

    let mut seal_trait_path = Path::from(Ident::new("__seal", Span::call_site()));
    seal_trait_path.segments.push(PathSegment::from(seal_ident));


    let tokens = quote! {

    };

    tokens.into()
}
