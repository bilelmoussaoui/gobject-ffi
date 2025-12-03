use quote::quote;
use syn::{Attribute, Type, parse::Parse};

use crate::types::CTypeOverride;

pub fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
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

pub fn extract_result_ok_type_as_type(ty: &Type) -> Type {
    let Type::Path(type_path) = ty else {
        return syn::parse_quote! { () };
    };

    let Some(segment) = type_path.path.segments.last() else {
        return syn::parse_quote! { () };
    };

    if segment.ident != "Result" {
        return syn::parse_quote! { () };
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return syn::parse_quote! { () };
    };

    if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
        ok_type.clone()
    } else {
        syn::parse_quote! { () }
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

pub(crate) fn extract_c_return_type(attrs: &[Attribute]) -> syn::Result<Option<CTypeOverride>> {
    extract_attribute(attrs, "c_return_type")
}

pub(crate) fn extract_c_type(attrs: &[Attribute]) -> syn::Result<Option<CTypeOverride>> {
    extract_attribute(attrs, "c_type")
}

fn extract_attribute<T: Parse>(attrs: &[Attribute], name: &str) -> syn::Result<Option<T>> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident(name))
        .map(|attr| attr.parse_args::<T>())
        .transpose()
}

pub(crate) fn check_fallibility(return_type: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = return_type {
        is_result_type(ty)
    } else {
        false
    }
}
