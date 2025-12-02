#![doc = include_str!("../../README.md")]

mod context;
mod generator;
mod params;
mod types;
mod utils;

use context::FfiMethodContext;
use generator::{AsyncFfiParams, FfiBuilder};
use params::{build_ffi_params, extract_c_return_type, extract_parameters};
use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ImplItemFn, ItemImpl, ReturnType, Type, parse_macro_input};
use types::FfiImplArgs;
use utils::extract_result_ok_type;

#[proc_macro_attribute]
pub fn c_return_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Generate FFI wrappers for all methods in an impl block
#[proc_macro_attribute]
pub fn ffi_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let args = parse_macro_input!(attr as FfiImplArgs);

    let self_type = &input.self_ty;
    let c_type_name_str = match args.get_c_type_name(self_type) {
        Ok(name) => name,
        Err(e) => return e.to_compile_error().into(),
    };
    let c_type_name = syn::Ident::new(&c_type_name_str, proc_macro2::Span::call_site());

    let type_name_lower = if let syn::Type::Path(type_path) = self_type.as_ref() {
        if let Some(last_segment) = type_path.path.segments.last() {
            // Convert to snake_case
            let name = last_segment.ident.to_string();
            // Simple snake_case conversion: insert _ before uppercase letters
            let mut snake = String::new();
            for (i, c) in name.chars().enumerate() {
                if c.is_uppercase() && i > 0 {
                    snake.push('_');
                }
                snake.push(c.to_ascii_lowercase());
            }
            snake
        } else {
            return syn::Error::new_spanned(
                self_type,
                "Cannot extract type name from this type. Expected a named type.",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(
            self_type,
            "Cannot extract type name from this type. Expected a path type (e.g., MyType, module::MyType)."
        ).to_compile_error().into();
    };
    let prefix = if args.prefix.value().is_empty() {
        type_name_lower
    } else {
        format!("{}_{}", args.prefix.value(), type_name_lower)
    };
    let ffi_type = args.ty;

    let mut ffi_functions = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let has_self = method
                .sig
                .inputs
                .iter()
                .any(|arg| matches!(arg, FnArg::Receiver(_)));

            if has_self {
                let generated =
                    generate_ffi_from_method(method, self_type, &c_type_name, &prefix, ffi_type);
                ffi_functions.push(generated);
            } else {
                let generated = generate_ffi_from_constructor(
                    method,
                    self_type,
                    &c_type_name,
                    &prefix,
                    ffi_type,
                );
                ffi_functions.push(generated);
            }
        }
    }

    let mut cleaned_input = input.clone();
    for item in &mut cleaned_input.items {
        if let ImplItem::Fn(method) = item {
            for arg in &mut method.sig.inputs {
                if let FnArg::Typed(pat_type) = arg {
                    pat_type
                        .attrs
                        .retain(|attr| !attr.path().is_ident("c_type"));
                }
            }
        }
    }

    // Generate type alias for GObject types
    let type_alias = if ffi_type.is_gobject() {
        // For GObject types, generate: pub type <TypeName>Ptr = <imp::<TypeName> as
        // ObjectSubclass>::Instance; We need to extract the type name from
        // self_type
        if let Type::Path(type_path) = self_type.as_ref() {
            if let Some(last_segment) = type_path.path.segments.last() {
                let type_name = &last_segment.ident;
                let ffi_type_name = &c_type_name;
                quote! {
                    pub type #ffi_type_name = <imp::#type_name as ::glib::subclass::prelude::ObjectSubclass>::Instance;
                }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    let get_type_fn = {
        let get_type_fn_name = syn::Ident::new(
            &format!("{}_get_type", prefix),
            proc_macro2::Span::call_site(),
        );
        quote! {
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #get_type_fn_name() -> ::glib::ffi::GType {
                ::glib::translate::IntoGlib::into_glib(<super::#self_type as ::glib::prelude::StaticType>::static_type())
            }
        }
    };

    let expanded = quote! {
        #cleaned_input

        pub mod ffi {
            use super::*;

            #type_alias

            #get_type_fn

            #(#ffi_functions)*
        }
    };

    TokenStream::from(expanded)
}

fn generate_ffi_from_constructor(
    method: &ImplItemFn,
    self_type: &Type,
    c_type_name: &syn::Ident,
    prefix: &str,
    ffi_type: types::FfiType,
) -> proc_macro2::TokenStream {
    let ctx = FfiMethodContext::new(method, prefix);
    let params = match extract_parameters(method, false) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };
    let c_return_type = match extract_c_return_type(&method.attrs) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };

    let ok_type = quote! { super::#self_type };
    let components = build_ffi_params(&params, &ok_type, c_type_name, c_return_type.as_ref(), true);

    let fn_name = &ctx.fn_name;
    let param_call_args = &components.param_call_args;
    let body = quote! { super::#self_type::#fn_name(#param_call_args) };

    if ctx.is_async {
        let sync_call_args = components.sync_call_args.clone();
        let async_params = AsyncFfiParams {
            c_type_name,
            ok_type: &ok_type,
            sync_call_args,
            kind: generator::FfiKind::Constructor,
            ffi_type,
        };

        FfiBuilder::from_context(ctx, components)
            .with_body(body)
            .build_async(async_params)
    } else {
        FfiBuilder::from_context(ctx, components)
            .with_body(body)
            .build_sync()
    }
}

fn generate_ffi_from_method(
    method: &ImplItemFn,
    self_type: &Type,
    c_type_name: &syn::Ident,
    prefix: &str,
    ffi_type: types::FfiType,
) -> proc_macro2::TokenStream {
    let ctx = FfiMethodContext::new(method, prefix);
    let params = match extract_parameters(method, true) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };
    let c_return_type = match extract_c_return_type(&method.attrs) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };

    let ok_type = match &method.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => {
            if ctx.is_fallible {
                extract_result_ok_type(ty)
            } else {
                quote! { #ty }
            }
        }
    };

    let components = build_ffi_params(
        &params,
        &ok_type,
        c_type_name,
        c_return_type.as_ref(),
        false,
    );

    // Determine self parameter type and conversion based on ffi_type
    let self_param_ident = syn::Ident::new("self_param", proc_macro2::Span::call_site());
    let self_type_in_ffi = syn::parse_quote! { super::#self_type };
    let (self_ffi_type, self_conversion) = if let Some(c_type) = ffi_type.self_c_type() {
        let transfer = ffi_type.self_transfer_mode();
        let conversion = transfer.convert_from(&self_param_ident, &self_type_in_ffi);
        (quote! { #c_type }, conversion)
    } else {
        let conversion =
            types::TransferMode::None.convert_from(&self_param_ident, &self_type_in_ffi);
        (quote! { *mut #c_type_name }, conversion)
    };

    let ffi_params = &components.params;
    let params_with_self = quote! {
        self_param: #self_ffi_type,
        #ffi_params
    };

    let mut conversions = vec![self_conversion];
    conversions.extend(components.conversions.clone());

    let fn_name = &ctx.fn_name;
    let param_call_args = &components.param_call_args;
    let body = quote! { self_param.#fn_name(#param_call_args) };

    if ctx.is_async {
        // Only clone self_param for GObject types (need it as source for Task)
        if ffi_type.is_gobject() {
            conversions.push(quote! {
                let source_for_task = self_param.clone();
            });
        }

        let sync_args = &components.sync_call_args;
        let sync_call_args = quote! {
            self_param,
            #sync_args
        };

        let async_params = AsyncFfiParams {
            c_type_name,
            ok_type: &ok_type,
            sync_call_args,
            kind: generator::FfiKind::Method {
                source_for_task: quote! { source_for_task },
            },
            ffi_type,
        };

        FfiBuilder::from_context(ctx, components)
            .with_body(body)
            .with_params(params_with_self)
            .with_args(conversions)
            .build_async(async_params)
    } else {
        FfiBuilder::from_context(ctx, components)
            .with_body(body)
            .with_params(params_with_self)
            .with_args(conversions)
            .build_sync()
    }
}
