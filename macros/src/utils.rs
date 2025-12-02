use quote::quote;
use syn::Type;

pub fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}

pub fn is_unit_type_token(tokens: &proc_macro2::TokenStream) -> bool {
    syn::parse2::<Type>(tokens.clone())
        .map(|ty| is_unit_type(&ty))
        .unwrap_or(false)
}

pub fn is_result_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Path(type_path) if type_path.path.segments.last()
            .is_some_and(|seg| seg.ident == "Result")
    )
}

pub fn extract_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
        Some(inner_type)
    } else {
        None
    }
}

pub fn is_mutable_reference(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if r.mutability.is_some())
}

pub fn extract_mut_ref_inner(ty: &Type) -> Option<&Type> {
    if let Type::Reference(r) = ty {
        if r.mutability.is_some() {
            return Some(&r.elem);
        }
    }
    None
}

pub fn extract_result_ok_type(ty: &Type) -> proc_macro2::TokenStream {
    let Type::Path(type_path) = ty else {
        return quote! { () };
    };

    let Some(segment) = type_path.path.segments.last() else {
        return quote! { () };
    };

    if segment.ident != "Result" {
        return quote! { () };
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return quote! { () };
    };

    if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
        quote! { #ok_type }
    } else {
        quote! { () }
    }
}

pub fn rust_type_to_c_type(ty: &Type) -> proc_macro2::TokenStream {
    if let Some(inner_type) = extract_mut_ref_inner(ty) {
        let inner_c_type = rust_type_to_c_type(inner_type);
        return quote! { *mut #inner_c_type };
    }

    if let Some(inner_type) = extract_option_inner(ty) {
        return quote! { <#inner_type as ::gobject_ffi::FfiConvert>::CType };
    }

    quote! { <#ty as ::gobject_ffi::FfiConvert>::CType }
}
