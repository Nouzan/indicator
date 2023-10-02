use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{
    FnArg, GenericParam, Ident, ItemFn, Lifetime, LifetimeParam, PatType, ReturnType, Type,
    Visibility,
};

/// Create a `RefOperator` from a function.
#[proc_macro_attribute]
pub fn operator(args: TokenStream, input: TokenStream) -> TokenStream {
    match generate_operator(args, input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn indicator() -> TokenStream2 {
    let found_crate = crate_name("indicator").expect("my-crate is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    }
}

fn generate_operator(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream2> {
    let indicator = indicator();
    let input_type = parse_input_type(args)?;
    let ItemFn {
        vis,
        sig,
        block,
        attrs,
    } = syn::parse::<ItemFn>(input)?;

    // Documentations.
    let docs = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect::<Vec<_>>();

    // Generate struct name.
    let fn_name = &sig.ident;
    let name = Ident::new(
        &format!("{}Op", fn_name.to_string().to_case(Case::Pascal)),
        Span::call_site(),
    );

    // Handle generics.
    let struct_def = generate_struct_def(&vis, &sig.generics, &name, &docs)?;
    let (orig_impl_generics, type_generics, where_clause) = sig.generics.split_for_impl();

    // Add lifetime to generics.
    let mut generics = sig.generics.clone();
    generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(Lifetime::new(
            "'value",
            Span::call_site(),
        ))));
    let (impl_generics, _, _) = generics.split_for_impl();

    // Handle extractors.
    let mut extractors = Vec::new();
    for arg in sig.inputs.iter() {
        let FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(arg, "expected typed argument"));
        };
        extractors.push(parse_extractor(arg)?);
    }

    // Handle output.
    let output = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let stmts = block.stmts;

    // Expand.
    Ok(quote! {
        #struct_def

        impl #impl_generics #indicator::context::RefOperator<'value, #input_type> for #name #type_generics #where_clause {
            type Output = #output;

            fn next(&mut self, __input: #indicator::context::ValueRef<'value, #input_type>) -> Self::Output {
                #(#extractors)*
                #(#stmts)*
            }
        }

        #(#docs)*
        #vis fn #fn_name #orig_impl_generics() -> #name #type_generics #where_clause {
            #name::default()
        }
    })
}

fn parse_input_type(args: TokenStream) -> syn::Result<Type> {
    syn::parse::<Type>(args)
}

fn parse_extractor(arg: &PatType) -> syn::Result<TokenStream2> {
    let indicator = indicator();
    let PatType { pat, ty, .. } = arg;
    Ok(quote! {
        let #pat: #ty = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
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
