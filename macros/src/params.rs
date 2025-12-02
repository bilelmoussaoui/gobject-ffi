use quote::quote;
use syn::{Attribute, FnArg, ImplItemFn, Pat, Type, parse::Parse};

use crate::{
    types::{CTypeOverride, TransferMode},
    utils::rust_type_to_c_type,
};

pub(crate) type FfiParam = (syn::Ident, Box<Type>, Option<CTypeOverride>);

pub(crate) struct FfiComponents {
    pub(crate) params: proc_macro2::TokenStream,
    pub(crate) conversions: Vec<proc_macro2::TokenStream>,
    pub(crate) return_type: proc_macro2::TokenStream,
    pub(crate) ok_handler: proc_macro2::TokenStream,
    pub(crate) err_handler: proc_macro2::TokenStream,
    pub(crate) sync_call_args: proc_macro2::TokenStream,
    pub(crate) param_call_args: proc_macro2::TokenStream,
}

pub(crate) fn extract_parameters(
    method: &ImplItemFn,
    skip_self: bool,
) -> syn::Result<Vec<FfiParam>> {
    let mut params = Vec::new();
    for arg in &method.sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                if skip_self {
                    continue;
                } else {
                    unreachable!("Constructor should not have self");
                }
            }
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    let c_type_override = extract_c_type(&pat_type.attrs)?;
                    params.push((param_name.clone(), pat_type.ty.clone(), c_type_override));
                }
            }
        }
    }
    Ok(params)
}

pub(crate) fn build_ffi_params(
    params: &[FfiParam],
    ok_type: &proc_macro2::TokenStream,
    c_type_name: &syn::Ident,
    c_return_type: Option<&CTypeOverride>,
    is_constructor: bool,
) -> FfiComponents {
    let mut ffi_params = quote! {};
    let mut conversions = Vec::new();

    for (param_name, param_type, c_type_override) in params {
        let c_param_type = if let Some(override_) = c_type_override {
            let c_type = &override_.c_type;
            quote! { #c_type }
        } else {
            rust_type_to_c_type(param_type)
        };
        ffi_params = quote! {
            #ffi_params
            #param_name: #c_param_type,
        };

        let conversion = generate_param_conversion(param_name, param_type, c_type_override);
        conversions.push(conversion);
    }

    let (finish_return, finish_ok_handler, finish_err_handler) =
        if is_constructor && c_return_type.is_none() {
            (
                quote! { *mut #c_type_name },
                TransferMode::Full.convert_to(quote! { val }),
                TransferMode::Full.error_value(),
            )
        } else {
            generate_return_type_handling(ok_type, c_return_type)
        };

    let sync_call_args = if params.is_empty() {
        quote! {}
    } else {
        let param_names: Vec<_> = params.iter().map(|(name, _, _)| name).collect();
        quote! {
            #(#param_names,)*
        }
    };

    let param_call_args = if params.is_empty() {
        quote! {}
    } else {
        let param_names: Vec<_> = params.iter().map(|(name, _, _)| name).collect();
        quote! {
            #(#param_names),*
        }
    };

    FfiComponents {
        params: ffi_params,
        conversions,
        return_type: finish_return,
        ok_handler: finish_ok_handler,
        err_handler: finish_err_handler,
        sync_call_args,
        param_call_args,
    }
}

fn generate_param_conversion(
    param_name: &syn::Ident,
    ty: &Type,
    c_type_override: &Option<CTypeOverride>,
) -> proc_macro2::TokenStream {
    if crate::utils::is_mutable_reference(ty) {
        return quote! {
            let #param_name: #ty = unsafe { &mut *#param_name };
        };
    }

    if let Some(override_) = c_type_override {
        override_.transfer.convert_from(param_name, ty)
    } else {
        quote! {
            let #param_name: #ty = unsafe { <#ty as ::gobject_ffi::FfiConvert>::from_c_borrowed(#param_name) };
        }
    }
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

fn generate_return_type_handling(
    ok_type: &proc_macro2::TokenStream,
    c_return_type: Option<&CTypeOverride>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    if crate::utils::is_unit_type_token(ok_type) {
        return (quote! { () }, quote! { () }, quote! { () });
    }

    let (c_type, ok_handler, error_value) = if let Some(override_) = c_return_type {
        let c_type = &override_.c_type;
        let ok_handler = override_.transfer.convert_to(quote! { val });
        let error_value = override_.transfer.error_value();
        (quote! { #c_type }, ok_handler, error_value)
    } else {
        let c_type = if let Ok(ty) = syn::parse2::<Type>(ok_type.clone()) {
            if let Some(inner_type) = crate::utils::extract_option_inner(&ty) {
                quote! { <#inner_type as ::gobject_ffi::FfiConvert>::CType }
            } else {
                quote! { <#ok_type as ::gobject_ffi::FfiConvert>::CType }
            }
        } else {
            quote! { <#ok_type as ::gobject_ffi::FfiConvert>::CType }
        };

        (
            c_type,
            quote! { <#ok_type as ::gobject_ffi::FfiConvert>::to_c_owned(val) },
            quote! { <#ok_type as ::gobject_ffi::FfiConvert>::c_error_value() },
        )
    };

    (c_type, ok_handler, error_value)
}
