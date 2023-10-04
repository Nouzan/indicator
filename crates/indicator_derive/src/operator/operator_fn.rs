use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, GenericParam, Generics, ItemFn, Result, ReturnType, Type, TypeTuple, Visibility,
};

use crate::operator::args::GenerateOut;

use super::{
    args::OperatorArgs,
    extractor::{Extractor, Extractors},
    utils::get_type_inside_option,
};

pub(super) struct OperatorFn {
    pub(super) vis: Visibility,
    pub(super) docs: Vec<Attribute>,
    pub(super) name: Ident,
    pub(super) extractors: Vec<Extractor>,
    pub(super) generics: Generics,
    pub(super) output_ty: Type,
    pub(super) next_fn: ItemFn,
}

impl OperatorFn {
    pub(super) fn parse_with(input: TokenStream, options: &OperatorArgs) -> Result<Self> {
        let mut original: ItemFn = syn::parse2(input)?;

        // Extract visibility.
        let mut vis = Visibility::Inherited;
        core::mem::swap(&mut original.vis, &mut vis);

        // Extract docs.
        let docs = original
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect::<Vec<_>>();
        original.attrs.clear();

        // Extract name.
        let mut name = Ident::new("__next", Span::call_site());
        core::mem::swap(&mut original.sig.ident, &mut name);

        // Collect the parsed generics.
        let mut generics = original.sig.generics.clone();

        // Parse extractors.
        let Extractors {
            generics: extra,
            extractors,
        } = Extractors::parse_with(&mut original.sig.inputs, options)?;
        generics.params.extend(extra);

        // Parse output.
        let OutputTy {
            generics: extra,
            ty: output_ty,
        } = OutputTy::parse_with(&original.sig.output, options)?;
        generics.params.extend(extra);

        Ok(Self {
            vis,
            docs,
            name,
            generics,
            output_ty,
            next_fn: original,
            extractors,
        })
    }
}

struct OutputTy {
    generics: Vec<GenericParam>,
    ty: Type,
}

impl OutputTy {
    fn parse_with(output: &ReturnType, options: &OperatorArgs) -> Result<Self> {
        fn get_ty(rt: &ReturnType) -> Type {
            match rt {
                ReturnType::Default => Type::Tuple(TypeTuple {
                    paren_token: Default::default(),
                    elems: Default::default(),
                }),
                ReturnType::Type(_, ty) => (**ty).clone(),
            }
        }

        let mut generics = Vec::new();

        let mut ty = get_ty(output);
        match options.generate_out {
            Some(GenerateOut::Out) => {
                generics.push(syn::parse2(quote!(OutTy: From<#ty>))?);
                ty = syn::parse2(quote!(OutTy))?;
            }
            Some(GenerateOut::Data) => {
                let data_ty = get_type_inside_option(&ty)?;
                generics.push(syn::parse2(quote!(DataTy: From<#data_ty>))?);
                ty = syn::parse2(quote!(Option<DataTy>))?;
            }
            Some(GenerateOut::WithData) => {
                let Type::Tuple(tuple) = ty else {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "the return type must be of the form `(_, Option<_>)`",
                    ));
                };
                if tuple.elems.len() != 2 {
                    return Err(syn::Error::new_spanned(
                        tuple,
                        "the return type must be of the form `(_, Option<_>)`",
                    ));
                }
                let out_ty = &tuple.elems[0];
                generics.push(syn::parse2(quote!(OutTy: From<#out_ty>))?);
                let data_ty = get_type_inside_option(&tuple.elems[1])?;
                generics.push(syn::parse2(quote!(DataTy: From<#data_ty>))?);
                ty = syn::parse2(quote!((OutTy, Option<DataTy>)))?;
            }
            _ => {}
        }
        Ok(Self { generics, ty })
    }
}
