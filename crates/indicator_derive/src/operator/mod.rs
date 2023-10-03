use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, FnArg, GenericParam, Ident, ItemFn, Lifetime, LifetimeParam, PatType,
    ReturnType, Token, Type, Visibility,
};

use self::args::{GenerateOut, OperatorArgs};
use super::indicator;

/// Arguments for generating operator.
mod args;

/// Expand the signature.
mod signature;

pub(super) fn generate_operator(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream2> {
    let args = syn::parse::<OperatorArgs>(args)?;
    let mut next = syn::parse::<ItemFn>(input)?;

    let next_fn = generate_next_fn(&next)?;
    signature::expand(&mut next.sig, &args)?;

    // Documentations.
    let docs = next
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect::<Vec<_>>();

    // Generate struct name.
    let fn_name = &next.sig.ident;
    let name = Ident::new(
        &format!("{}Op", fn_name.to_string().to_case(Case::Pascal)),
        Span::call_site(),
    );

    // Handle generics.
    let struct_def = generate_struct_def(&next.vis, &next.sig.generics, &name, &docs)?;
    let (orig_impl_generics, type_generics, where_clause) = next.sig.generics.split_for_impl();

    // Add lifetime to generics.
    let mut generics = next.sig.generics.clone();
    generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
            "'value",
            Span::call_site(),
        ))));
    let (impl_generics, _, _) = generics.split_for_impl();

    // Handle extractors.
    let mut extractors = Punctuated::<_, Token![,]>::new();
    for arg in next.sig.inputs.iter() {
        let FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(arg, "expected typed argument"));
        };
        extractors.push(parse_extractor(arg)?);
    }

    // Handle output.
    let output = match &next.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let indicator = indicator();
    let vis = &next.vis;
    let input_type = &args.input_type;
    let return_stmt = match args.generate_out {
        Some(GenerateOut::Out) => {
            quote! {
                __next(#extractors).into()
            }
        }
        Some(GenerateOut::Data) => {
            quote! {
                __next(#extractors).map(Into::into)
            }
        }
        Some(GenerateOut::WithData) => {
            quote! {
                {
                    let (__out, __data) = __next(#extractors);
                    (__out.into(), __data.map(Into::into))
                }
            }
        }
        None => {
            quote! {
                __next(#extractors)
            }
        }
    };

    // Expand.
    Ok(quote! {
        #struct_def

        impl #impl_generics #indicator::context::RefOperator<'value, #input_type> for #name #type_generics #where_clause {
            type Output = #output;

            fn next(&mut self, __input: #indicator::context::ValueRef<'value, #input_type>) -> Self::Output {
                #next_fn
                #return_stmt
            }
        }

        #(#docs)*
        #vis fn #fn_name #orig_impl_generics() -> #name #type_generics #where_clause {
            #name::default()
        }
    })
}

fn generate_next_fn(next: &ItemFn) -> syn::Result<TokenStream2> {
    let mut next = next.clone();
    next.vis = Visibility::Inherited;
    next.sig.ident = Ident::new("__next", next.sig.ident.span());
    Ok(quote! {
        #next
    })
}

fn parse_extractor(arg: &PatType) -> syn::Result<TokenStream2> {
    let indicator = indicator();
    let PatType { ty, .. } = arg;
    Ok(quote! {
        {
            let __a: #ty = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
            __a
        }
    })
}

fn generate_struct_def(
    vis: &Visibility,
    generics: &syn::Generics,
    name: &syn::Ident,
    docs: &[syn::Attribute],
) -> syn::Result<TokenStream2> {
    if generics.params.is_empty() {
        return Ok(quote! {
            #[derive(Default)]
            #[allow(non_camel_case_types)]
            #vis struct #name;
        });
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let phantom_data_type = generate_phantom_data_type(generics);
    Ok(quote! {
        #(#docs)*
        #[allow(non_camel_case_types)]
        #vis struct #name #impl_generics (core::marker::PhantomData<#phantom_data_type> ) #where_clause;

        impl #impl_generics Default for #name #type_generics #where_clause {
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }
    })
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
