use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, FnArg, GenericParam, Meta, Pat, PatType, Result, Token, Type,
};

use quote::quote;

use super::{
    args::OperatorArgs,
    indicator,
    utils::{get_type_inside_option, get_type_under_reference},
};

pub(super) struct Extractors {
    pub(super) generics: Vec<GenericParam>,
    pub(super) extractors: Vec<Extractor>,
}

impl Extractors {
    /// Parse extractors from the function inputs and clear the attributes.
    pub(super) fn parse_with(
        args: &mut Punctuated<FnArg, Comma>,
        options: &OperatorArgs,
    ) -> Result<Self> {
        let mut ctx = Ctx::default();
        let mut generics = Vec::default();
        let mut extractors = Vec::default();
        for arg in args.iter_mut() {
            let FnArg::Typed(arg) = arg else {
                return Err(syn::Error::new(arg.span(), "Expecting a typed argument"));
            };
            let ExtractorWithGenerics {
                generics: extra,
                extractor,
            } = ExtractorWithGenerics::parse_with(arg, options, &mut ctx)?;
            generics.extend(extra);
            extractors.push(extractor);
            arg.attrs.clear();
        }
        Ok(Self {
            generics,
            extractors,
        })
    }
}

#[derive(Default)]
struct Ctx {
    input: usize,
    env: usize,
    data: usize,
    prev: usize,
}

impl Ctx {
    fn next_input(&mut self, pat: &Pat) -> String {
        let name = get_variable_name(pat).map(|ident| ident.to_string());
        let name = name.unwrap_or_else(|| format!("input{}", self.input));
        self.input += 1;
        name
    }

    fn next_env(&mut self, pat: &Pat) -> String {
        let name = get_variable_name(pat).map(|ident| ident.to_string());
        let name = name.unwrap_or_else(|| format!("env{}", self.input));
        self.env += 1;
        name
    }

    fn next_data(&mut self, pat: &Pat) -> String {
        let name = get_variable_name(pat).map(|ident| ident.to_string());
        let name = name.unwrap_or_else(|| format!("data{}", self.input));
        self.data += 1;
        name
    }

    fn next_prev(&mut self, pat: &Pat) -> String {
        let name = get_variable_name(pat).map(|ident| ident.to_string());
        let name = name.unwrap_or_else(|| format!("prev{}", self.input));
        self.prev += 1;
        name
    }
}

fn get_variable_name(pat: &Pat) -> Option<&Ident> {
    match pat {
        Pat::Ident(pat) => Some(&pat.ident),
        _ => None,
    }
}

type Optional = bool;

pub(super) enum Extractor {
    Plain(Box<Type>),
    Borrow(Ident, FnArg, Optional),
    AsRef(Ident, FnArg, Optional),
}

impl Extractor {
    pub(super) fn expand(&self) -> TokenStream {
        let indicator = indicator();
        match self {
            Self::Plain(ty) => {
                quote! {
                    {
                        let __a: #ty = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
                        __a
                    }
                }
            }
            Self::Borrow(name, arg, false) => {
                quote! {
                    {
                        let #arg = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
                        core::borrow::Borrow::borrow(#name)
                    }
                }
            }
            Self::Borrow(name, arg, true) => {
                quote! {
                    {
                        let #arg = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
                        #name.map(core::borrow::Borrow::borrow)
                    }
                }
            }
            Self::AsRef(name, arg, false) => {
                quote! {
                    {
                        let #arg = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
                        core::convert::AsRef::as_ref(#name)
                    }
                }
            }
            Self::AsRef(name, arg, true) => {
                quote! {
                    {
                        let #arg = #indicator::context::extractor::FromValueRef::from_value_ref(&__input);
                        #name.map(core::convert::AsRef::as_ref)
                    }
                }
            }
        }
    }
}

struct ExtractorWithGenerics {
    generics: Vec<GenericParam>,
    extractor: Extractor,
}

impl ExtractorWithGenerics {
    fn parse_with(arg: &PatType, options: &OperatorArgs, ctx: &mut Ctx) -> Result<Self> {
        let indicator = indicator();
        let mut generics = Vec::new();
        let extractor = match arg.attrs.len() {
            0 => Extractor::Plain(arg.ty.clone()),
            1 => {
                match Attr::parse(&arg.attrs[0])? {
                    Attr::Input(way) => {
                        let target_ty = get_type_under_reference(&arg.ty)?;
                        let ty = &options.input_type;
                        let name = Ident::new(&ctx.next_input(&arg.pat), arg.span());
                        let pat = parse_quote!(#indicator::context::In(#name): #indicator::context::In<&#ty>);
                        if way.is_borrow() {
                            generics
                                .push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty>))?);
                            Extractor::Borrow(name, pat, false)
                        } else {
                            generics.push(syn::parse2(quote!(#ty: AsRef<#target_ty>))?);
                            Extractor::AsRef(name, pat, false)
                        }
                    }
                    Attr::Env(way, false) => {
                        let target_ty = get_type_under_reference(&arg.ty)?;
                        let name = ctx.next_env(&arg.pat);
                        let ty = Ident::new(&name.to_case(Case::Pascal), arg.span());
                        let name = Ident::new(&name, arg.span());
                        let pat = parse_quote!(#indicator::context::Env(#name): #indicator::context::Env<&#ty>);
                        if way.is_borrow() {
                            generics.push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static))?);
                            Extractor::Borrow(name, pat, false)
                        } else {
                            generics.push(syn::parse2(
                                quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static),
                            )?);
                            Extractor::AsRef(name, pat, false)
                        }
                    }
                    Attr::Env(way, true) => {
                        let target_ref = get_type_inside_option(&arg.ty)?;
                        let target_ty = get_type_under_reference(target_ref)?;
                        let name = ctx.next_env(&arg.pat);
                        let ty = Ident::new(&name.to_case(Case::Pascal), arg.span());
                        let name = Ident::new(&name, arg.span());
                        let pat = parse_quote!(#indicator::context::Env(#name): #indicator::context::Env<Option<&#ty>>);
                        if way.is_borrow() {
                            generics.push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static))?);
                            Extractor::Borrow(name, pat, true)
                        } else {
                            generics.push(syn::parse2(
                                quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static),
                            )?);
                            Extractor::AsRef(name, pat, true)
                        }
                    }
                    Attr::Data(way, false) => {
                        let target_ty = get_type_under_reference(&arg.ty)?;
                        let name = ctx.next_data(&arg.pat);
                        let ty = Ident::new(&name.to_case(Case::Pascal), arg.span());
                        let name = Ident::new(&name, arg.span());
                        let pat = parse_quote!(#indicator::context::Data(#name): #indicator::context::Data<&#ty>);
                        if way.is_borrow() {
                            generics.push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static))?);
                            Extractor::Borrow(name, pat, false)
                        } else {
                            generics.push(syn::parse2(
                                quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static),
                            )?);
                            Extractor::AsRef(name, pat, false)
                        }
                    }
                    Attr::Data(way, true) => {
                        let target_ref = get_type_inside_option(&arg.ty)?;
                        let target_ty = get_type_under_reference(target_ref)?;
                        let name = ctx.next_data(&arg.pat);
                        let ty = Ident::new(&name.to_case(Case::Pascal), arg.span());
                        let name = Ident::new(&name, arg.span());
                        let pat = parse_quote!(#indicator::context::Data(#name): #indicator::context::Data<Option<&#ty>>);
                        if way.is_borrow() {
                            generics.push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static))?);
                            Extractor::Borrow(name, pat, true)
                        } else {
                            generics.push(syn::parse2(
                                quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static),
                            )?);
                            Extractor::AsRef(name, pat, true)
                        }
                    }
                    Attr::Prev(way) => {
                        let target_ref = get_type_inside_option(&arg.ty)?;
                        let target_ty = get_type_under_reference(target_ref)?;
                        let name = ctx.next_prev(&arg.pat);
                        let ty = Ident::new(&name.to_case(Case::Pascal), arg.span());
                        let name = Ident::new(&name, arg.span());
                        let pat = parse_quote!(#indicator::context::Prev(#name): #indicator::context::Prev<&#ty>);
                        if way.is_borrow() {
                            generics.push(syn::parse2(quote!(#ty: core::borrow::Borrow<#target_ty> + Send + Sync + 'static))?);
                            Extractor::Borrow(name, pat, true)
                        } else {
                            generics.push(syn::parse2(
                                quote!(#ty: AsRef<#target_ty> + Send + Sync + 'static),
                            )?);
                            Extractor::AsRef(name, pat, true)
                        }
                    }
                }
            }
            _ => {
                return Err(syn::Error::new(
                    arg.span(),
                    "Expecting at most one attribute",
                ));
            }
        };
        Ok(Self {
            generics,
            extractor,
        })
    }
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

struct WayWithOptional {
    way: Way,
    question_mark: Option<Token![?]>,
}

impl Parse for WayWithOptional {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let way = match ident.to_string().as_str() {
            "borrow" => Way::Borrow,
            "as_ref" => Way::AsRef,
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "Expecting `borrow` or `as_ref`",
                ));
            }
        };
        if input.peek(Token![?]) {
            Ok(Self {
                way,
                question_mark: Some(input.parse()?),
            })
        } else {
            Ok(Self {
                way,
                question_mark: None,
            })
        }
    }
}

enum Attr {
    Input(Way),
    Env(Way, Optional),
    Data(Way, Optional),
    Prev(Way),
}

impl Attr {
    fn parse(attr: &Attribute) -> Result<Self> {
        let (ident, tokens) = match &attr.meta {
            Meta::Path(path) => {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new(attr.span(), "Expecting an identifier"))?;
                (ident, None)
            }
            Meta::List(list) => {
                let ident = list
                    .path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new(attr.span(), "Expecting an identifier"))?;
                (ident, Some(&list.tokens))
            }
            _ => return Err(syn::Error::new(attr.span(), "Unsupported attribute format")),
        };
        let (way, optional) = tokens
            .map(|tokens| syn::parse2(tokens.clone()))
            .transpose()?
            .map(|WayWithOptional { way, question_mark }| (way, question_mark.is_some()))
            .unwrap_or_else(|| (Way::Borrow, false));
        match ident.to_string().as_str() {
            "input" => {
                if optional {
                    Err(syn::Error::new(
                        attr.span(),
                        "The `input` attribute cannot be optional",
                    ))
                } else {
                    Ok(Self::Input(way))
                }
            }
            "env" => Ok(Self::Env(way, optional)),
            "data" => Ok(Self::Data(way, optional)),
            "prev" => {
                if optional {
                    Err(syn::Error::new(
                        attr.span(),
                        "The `prev` attribute is always optional",
                    ))
                } else {
                    Ok(Self::Prev(way))
                }
            }
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown attribute: `{ident}`, expecting `input`, `env`, `data` or `prev`",),
            )),
        }
    }
}
