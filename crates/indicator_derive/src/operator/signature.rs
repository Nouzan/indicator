use quote::quote;
use syn::{Result, ReturnType, Signature, Type, TypeTuple};

use super::args::{GenerateOut, OperatorArgs};

pub(super) fn expand(sig: &mut Signature, args: &OperatorArgs) -> Result<()> {
    expand_generics(sig, args)?;
    Ok(())
}

fn expand_generics(sig: &mut Signature, args: &OperatorArgs) -> Result<()> {
    match args.generate_out {
        Some(GenerateOut::Out) => {
            let ty = get_return_type(sig);
            sig.generics
                .params
                .push(syn::parse2(quote!(OutTy: From<#ty>))?);
            sig.output = syn::parse2(quote!(-> OutTy))?;
        }
        Some(GenerateOut::Data) => {
            let ty = get_return_type(sig);
            let data_ty = get_type_inside_option(&ty)?;
            sig.generics
                .params
                .push(syn::parse2(quote!(DataTy: From<#data_ty>))?);
            sig.output = syn::parse2(quote!(-> Option<DataTy>))?;
        }
        Some(GenerateOut::WithData) => {
            let ty = get_return_type(sig);
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
            sig.generics
                .params
                .push(syn::parse2(quote!(OutTy: From<#out_ty>))?);
            let data_ty = get_type_inside_option(&tuple.elems[1])?;
            sig.generics
                .params
                .push(syn::parse2(quote!(DataTy: From<#data_ty>))?);
            sig.output = syn::parse2(quote!(-> (OutTy, Option<DataTy>)))?;
        }
        _ => {}
    }
    Ok(())
}

fn get_return_type(sig: &Signature) -> Type {
    match &sig.output {
        ReturnType::Default => Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Default::default(),
        }),
        ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

fn get_type_inside_option(ty: &Type) -> Result<&Type> {
    let path = match ty {
        Type::Path(path) => path,
        _ => return Err(syn::Error::new_spanned(ty, "expected `Option<_>`")),
    };
    let segment = path.path.segments.last().unwrap();
    if segment.ident != "Option" {
        return Err(syn::Error::new_spanned(ty, "expected `Option<_>`"));
    }
    let args = &segment.arguments;
    let args = match args {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return Err(syn::Error::new_spanned(ty, "expected `Option<_>`")),
    };
    let arg = args.args.first().unwrap();
    let arg = match arg {
        syn::GenericArgument::Type(ty) => ty,
        _ => return Err(syn::Error::new_spanned(ty, "expected `Option<_>`")),
    };
    Ok(arg)
}
