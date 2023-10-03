use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, ExprAssign, ExprPath, Ident, Result, Token, Type, TypePath,
};

pub(crate) struct OperatorArgs {
    pub(crate) input_type: Type,
    pub(crate) generate_out: Option<GenerateOut>,
}

pub(crate) enum GenerateOut {
    Out,
    WithData,
    Data,
}

impl Parse for OperatorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let args = Punctuated::<Args, Token![,]>::parse_terminated(input)?;
        let mut input_type = None;
        let mut generate_out = None;
        for arg in args {
            match arg {
                Args::GenerateOut(out) => generate_out = Some(out),
                Args::InputType(ty) => input_type = Some(ty),
            }
        }
        let input_type = input_type
            .ok_or_else(|| syn::Error::new(Span::call_site(), "`input` argument is required"))?;
        Ok(Self {
            input_type,
            generate_out,
        })
    }
}

enum Args {
    InputType(Type),
    GenerateOut(GenerateOut),
}

fn get_ident(expr: &Expr) -> Result<&Ident> {
    let Expr::Path(ExprPath { path, .. }) = expr else {
        return Err(syn::Error::new(
            Span::call_site(),
            "Expecting an identifier",
        ));
    };
    let ident = path
        .get_ident()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "Expecting an identifier"))?;
    Ok(ident)
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr: Expr = input.parse()?;
        match expr {
            Expr::Assign(ExprAssign { left, right, .. }) => {
                let ident = get_ident(&left)?;
                if ident == "input" {
                    let Expr::Path(ExprPath { path, .. }) = *right else {
                        return Err(syn::Error::new(Span::call_site(), "Expecting a type"));
                    };
                    Ok(Self::InputType(Type::Path(TypePath { qself: None, path })))
                } else {
                    Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown argument: `{ident}`, expecting `input = T`"),
                    ))
                }
            }
            expr => {
                let ident = get_ident(&expr)?;
                match ident.to_string().as_str() {
                    "generate_out" => Ok(Self::GenerateOut(GenerateOut::Out)),
                    "generate_data" => Ok(Self::GenerateOut(GenerateOut::Data)),
                    "generate_out_with_data" => Ok(Self::GenerateOut(GenerateOut::WithData)),
                    _ => Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown argument: `{ident}`, expecting `generate_out`, `generate_out_data` or `generate_out_with_data`"),
                    )),
                }
            }
        }
    }
}
