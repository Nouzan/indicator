use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, Attribute, GenericParam, Generics, ItemFn, Result, ReturnType, Token,
    Type, TypeTuple, Visibility,
};

use crate::{indicator, operator::args::GenerateOut};

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
    input_ty: Type,
    out: Option<GenerateOut>,
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
            input_ty: options.input_type.clone(),
            out: options.generate_out,
        })
    }

    pub(super) fn expand(&self) -> TokenStream {
        let indicator = indicator();
        let vis = &self.vis;
        let docs = &self.docs;
        let struct_def = self.expand_struct();
        let name = self.struct_name();
        let fn_name = &self.name;
        let (orig_impl_generics, type_generics, where_clause) = self.generics.split_for_impl();
        let input_ty = &self.input_ty;
        let output_ty = &self.output_ty;
        let next_fn = &self.next_fn;

        let gen = self.generics_with_lifetime();
        let impl_generics = gen.split_for_impl().0;

        let return_stmt = self.expand_stmt();
        quote! {
            #struct_def

            impl #impl_generics #indicator::context::RefOperator<'value, #input_ty> for #name #type_generics #where_clause {
                type Output = #output_ty;

                fn next(&mut self, __input: #indicator::context::ValueRef<'value, #input_ty>) -> Self::Output {
                    #[allow(clippy::extra_unused_type_parameters)]
                    #next_fn
                    #return_stmt
                }
            }

            #(#docs)*
            #vis fn #fn_name #orig_impl_generics() -> #name #type_generics #where_clause {
                #name::default()
            }
        }
    }

    fn struct_name(&self) -> Ident {
        Ident::new(
            &format!("{}Op", self.name.to_string().to_case(Case::Pascal)),
            self.name.span(),
        )
    }

    fn generics_with_lifetime(&self) -> Generics {
        let mut generics = self.generics.clone();
        generics.params.push(syn::parse_quote!('value));
        generics
    }

    fn expand_struct(&self) -> TokenStream {
        let vis = &self.vis;
        let name = self.struct_name();
        let generics = &self.generics;
        let docs = &self.docs;

        if generics.params.is_empty() {
            return quote! {
                #[derive(Default)]
                #[allow(non_camel_case_types)]
                #vis struct #name;
            };
        }
        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        let phantom_data_type = generate_phantom_data_type(generics);
        quote! {
            #(#docs)*
            #[allow(non_camel_case_types)]
            #vis struct #name #impl_generics (core::marker::PhantomData<#phantom_data_type> ) #where_clause;

            impl #impl_generics Default for #name #type_generics #where_clause {
                fn default() -> Self {
                    Self(core::marker::PhantomData)
                }
            }
        }
    }

    fn expand_call_name(&self) -> TokenStream {
        let call_generics = if self.next_fn.sig.generics.params.is_empty() {
            quote!()
        } else {
            let (_, type_generics, _) = self.next_fn.sig.generics.split_for_impl();
            quote!(::#type_generics)
        };
        quote! {
            __next #call_generics
        }
    }

    fn expand_extractors(&self) -> Punctuated<TokenStream, Token![,]> {
        self.extractors
            .iter()
            .map(|extractor| extractor.expand())
            .collect()
    }

    fn expand_stmt(&self) -> TokenStream {
        let call_name = self.expand_call_name();
        let extractors = self.expand_extractors();
        match self.out {
            Some(GenerateOut::Out) => {
                quote! {
                    ((#call_name (#extractors)).into(), None)
                }
            }
            Some(GenerateOut::Data) => {
                quote! {
                    #call_name (#extractors).map(Into::into)
                }
            }
            Some(GenerateOut::WithData) => {
                quote! {
                    {
                        let (__out, __data) = #call_name (#extractors);
                        (__out.into(), __data.map(Into::into))
                    }
                }
            }
            None => {
                quote! {
                    #call_name (#extractors)
                }
            }
        }
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
                ty = syn::parse2(quote!((OutTy, Option<()>)))?;
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

fn generate_phantom_data_type(generics: &syn::Generics) -> Type {
    let params: Vec<_> = generics
        .params
        .iter()
        .filter_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                let ident = &type_param.ident;
                Some(quote! { #ident })
            } else {
                None
            }
        })
        .collect();

    // Generate `fn() -> (generics)` type
    let phantom_data_type = quote! { fn() -> (#(#params),*) };

    // Parse as `fn() -> (generics)` type
    let phantom_data_type: Type = syn::parse2(phantom_data_type).unwrap();
    phantom_data_type
}
