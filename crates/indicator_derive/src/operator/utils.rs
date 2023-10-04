use syn::{Result, Type, TypeReference};

pub(super) fn get_type_inside_option(ty: &Type) -> Result<&Type> {
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

pub(super) fn get_type_under_reference(ty: &Type) -> Result<&Type> {
    let Type::Reference(TypeReference {
        elem: target_ty,
        lifetime: None,
        mutability: None,
        ..
    }) = ty
    else {
        return Err(syn::Error::new_spanned(
            ty,
            "expected reference type without lifetime and mutability, e.g. `&T`",
        ));
    };
    Ok(&*target_ty)
}
