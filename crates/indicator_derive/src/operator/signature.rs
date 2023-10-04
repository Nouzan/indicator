use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    FnArg, ItemFn, Meta, Pat, Result, ReturnType, Signature, Type, TypeReference, TypeTuple,
};

use super::args::{GenerateOut, OperatorArgs};

pub(super) fn expand(input: &mut ItemFn, args: &OperatorArgs) -> Result<ItemFn> {
    let mut unattributed = input.clone();
    remove_input_attributes(&mut unattributed)?;
    expand_generics(&mut input.sig, args)?;
    expand_inputs(&mut input.sig, args)?;
    Ok(unattributed)
}

fn remove_input_attributes(item_fn: &mut ItemFn) -> Result<()> {
    for arg in item_fn.sig.inputs.iter_mut() {
        let FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(arg, "expected typed argument"));
        };
        arg.attrs.clear();
    }
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

fn expand_inputs(sig: &mut Signature, args: &OperatorArgs) -> Result<()> {
    let mut generics = vec![];
    let mut ctx = Ctx::default();
    for arg in sig.inputs.iter_mut() {
        if let Some(generic) = expand_input(&mut ctx, arg, &args.input_type)? {
            generics.push(generic);
        }
    }
    if !generics.is_empty() {
        sig.generics.params.extend(generics);
    }
    Ok(())
}

#[derive(Default)]
struct Ctx {
    input: usize,
    env: usize,
    data: usize,
}

enum Way {
    Borrow,
    AsRef,
}

impl Way {
    fn is_borrow(&self) -> bool {
        matches!(self, Way::Borrow)
    }
}

enum ArgKind {
    Input(Way),
    Env(Way),
    Data(Way),
    Prev(Way),
}

impl<'a> TryFrom<&'a Meta> for ArgKind {
    type Error = syn::Error;

    fn try_from(value: &'a Meta) -> std::result::Result<Self, Self::Error> {
        match value {
            Meta::Path(path) => {
                let ident = path.get_ident().ok_or_else(|| {
                    syn::Error::new_spanned(
                        path,
                        "unsupported attribute, expected `input`, `env` or `data`",
                    )
                })?;
                match ident.to_string().as_str() {
                    "input" => Ok(ArgKind::Input(Way::Borrow)),
                    "env" => Ok(ArgKind::Env(Way::Borrow)),
                    "data" => Ok(ArgKind::Data(Way::Borrow)),
                    "prev" => Ok(ArgKind::Prev(Way::Borrow)),
                    _ => Err(syn::Error::new_spanned(
                        path,
                        "unsupported attribute, expected `input`, `env` or `data`",
                    )),
                }
            }
            Meta::List(list) => {
                let way: Ident = syn::parse2(list.tokens.clone())?;
                let way = match way.to_string().as_str() {
                    "borrow" => Way::Borrow,
                    "as_ref" => Way::AsRef,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            way,
                            "unsupported value, expected `borrow` or `as_ref`",
                        ))
                    }
                };
                let ident = list.path.get_ident().ok_or_else(|| {
                    syn::Error::new_spanned(
                        list,
                        "unsupported attribute, expected `input`, `env` or `data`",
                    )
                })?;
                match ident.to_string().as_str() {
                    "input" => Ok(ArgKind::Input(way)),
                    "env" => Ok(ArgKind::Env(way)),
                    "data" => Ok(ArgKind::Data(way)),
                    "prev" => Ok(ArgKind::Prev(way)),
                    _ => Err(syn::Error::new_spanned(
                        ident,
                        "unsupported attribute, expected `input`, `env` or `data`",
                    )),
                }
            }
            _ => Err(syn::Error::new_spanned(
                value,
                "unsupported attribute, expected `input`, `env` or `data`",
            )),
        }
    }
}

fn expand_input(
    ctx: &mut Ctx,
    fn_arg: &mut FnArg,
    input_ty: &Type,
) -> Result<Option<syn::GenericParam>> {
    let indicator = super::indicator();

    let FnArg::Typed(arg) = fn_arg else {
        return Err(syn::Error::new_spanned(fn_arg, "expected typed argument"));
    };
    let attr = arg.attrs.pop();
    if !arg.attrs.is_empty() {
        return Err(syn::Error::new_spanned(
            arg,
            "expected at most one attribute",
        ));
    }
    let Some(attr) = attr else {
        return Ok(None);
    };
    let Type::Reference(TypeReference {
        elem: target_ty,
        lifetime: None,
        mutability: None,
        ..
    }) = &*arg.ty
    else {
        return Err(syn::Error::new_spanned(
            arg,
            "expected reference type without lifetime and mutability, e.g. `&T`",
        ));
    };
    let kind: ArgKind = ArgKind::try_from(&attr.meta)?;
    let name = get_variable_name(&arg.pat).map(|n| n.to_string());
    let generic = match kind {
        ArgKind::Input(way) => {
            let name = Ident::new(
                &name.unwrap_or_else(|| format!("input{}", ctx.input)),
                Span::call_site(),
            );
            ctx.input += 1;
            let pat = quote!(#indicator::context::In(#name));
            let ty = input_ty.clone();
            let (generic, attr) = if way.is_borrow() {
                (
                    syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty>))?,
                    quote!(#[borrow(#name)]),
                )
            } else {
                (
                    syn::parse2(quote!(#ty: AsRef<#target_ty>))?,
                    quote!(#[as_ref(#name)]),
                )
            };
            *fn_arg = syn::parse2(quote!(#attr #pat: #indicator::context::In<&#ty>))?;
            generic
        }
        ArgKind::Env(way) => {
            let name_string = name.unwrap_or_else(|| format!("env{}", ctx.env));
            let name = Ident::new(&name_string, Span::call_site());
            ctx.env += 1;
            let pat = quote!(#indicator::context::Env(#name));
            let ty = Ident::new(&name_string.to_case(Case::Pascal), Span::call_site());
            let (generic, attr) = if way.is_borrow() {
                (
                    syn::parse2(
                        quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static),
                    )?,
                    quote!(#[borrow(#name)]),
                )
            } else {
                (
                    syn::parse2(quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static))?,
                    quote!(#[as_ref(#name)]),
                )
            };
            *fn_arg = syn::parse2(quote!(#attr #pat: #indicator::context::Env<&#ty>))?;
            generic
        }
        ArgKind::Data(way) => {
            let name_string = name.unwrap_or_else(|| format!("data{}", ctx.env));
            let name = Ident::new(&name_string, Span::call_site());
            ctx.data += 1;
            let pat = quote!(#indicator::context::Data(#name));
            let ty = Ident::new(&name_string.to_case(Case::Pascal), Span::call_site());
            let (generic, attr) = if way.is_borrow() {
                (
                    syn::parse2(
                        quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static),
                    )?,
                    quote!(#[borrow(#name)]),
                )
            } else {
                (
                    syn::parse2(quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static))?,
                    quote!(#[as_ref(#name)]),
                )
            };
            *fn_arg = syn::parse2(quote!(#attr #pat: #indicator::context::Data<&#ty>))?;
            generic
        }
        ArgKind::Prev(way) => {
            let name_string = name.unwrap_or_else(|| format!("prev{}", ctx.env));
            let name = Ident::new(&name_string, Span::call_site());
            ctx.data += 1;
            let pat = quote!(#indicator::context::Prev(#name));
            let ty = Ident::new(&name_string.to_case(Case::Pascal), Span::call_site());
            let (generic, attr) = if way.is_borrow() {
                (
                    syn::parse2(
                        quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static),
                    )?,
                    quote!(#[borrow(#name)]),
                )
            } else {
                (
                    syn::parse2(quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static))?,
                    quote!(#[as_ref(#name)]),
                )
            };
            *fn_arg = syn::parse2(quote!(#attr #pat: #indicator::context::Prev<&#ty>))?;
            generic
        }
    };
    Ok(Some(generic))
}

fn get_variable_name(pat: &Pat) -> Option<&Ident> {
    match pat {
        Pat::Ident(pat) => Some(&pat.ident),
        _ => None,
    }
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
