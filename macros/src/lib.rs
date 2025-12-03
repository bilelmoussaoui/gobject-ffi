#![doc = include_str!("../../README.md")]

mod method;
mod types;
mod utils;

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ItemImpl, Type, parse_macro_input};
use types::FfiImplArgs;

#[proc_macro_attribute]
pub fn c_return_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

fn generate_type_alias(
    ffi_type: types::FfiType,
    self_type: &Type,
    c_type_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    if let Type::Path(type_path) = self_type {
        if let Some(last_segment) = type_path.path.segments.last() {
            let type_name = &last_segment.ident;
            return match ffi_type {
                types::FfiType::Object => quote! {
                    pub type #c_type_name = <imp::#type_name as ::glib::subclass::prelude::ObjectSubclass>::Instance;
                },
                types::FfiType::Shared => quote! {
                    pub type #c_type_name = *mut <<super::#type_name as ::glib::subclass::shared::SharedType>::RefCountedType as ::glib::subclass::shared::RefCounted>::InnerType;
                },
                types::FfiType::Boxed => quote! {
                    pub type #c_type_name = <super::#type_name as ::glib::translate::GlibPtrDefault>::GlibType;
                },
                types::FfiType::Enum => quote! {
                    pub type #c_type_name = super::#type_name;
                },
                types::FfiType::Flags => quote! {
                    pub type #c_type_name = super::#type_name;
                },
            };
        }
    }
    quote! {}
}

fn generate_get_type_fn(prefix: &str, self_type: &Type) -> proc_macro2::TokenStream {
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
}

/// Generate FFI wrappers for all methods in an impl block
#[proc_macro_attribute]
pub fn ffi_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let args = parse_macro_input!(attr as FfiImplArgs);

    let self_type = &input.self_ty;

    let type_name_lower = if let syn::Type::Path(type_path) = self_type.as_ref() {
        if let Some(last_segment) = type_path.path.segments.last() {
            last_segment.ident.to_string().to_snake_case()
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
        type_name_lower.clone()
    } else {
        format!("{}_{}", args.prefix.value(), type_name_lower)
    };

    let c_type_name_str = if let Some(ref explicit_name) = args.c_type_name {
        explicit_name.value()
    } else {
        prefix.to_pascal_case()
    };
    let c_type_name = syn::Ident::new(&c_type_name_str, proc_macro2::Span::call_site());

    let ffi_type = args.ty;

    let mut ffi_functions = Vec::new();
    let mut headers = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            let has_self = method
                .sig
                .inputs
                .iter()
                .any(|arg| matches!(arg, FnArg::Receiver(_)));

            let is_constructor = !has_self;

            let ffi_method = match method::FfiMethod::from_method(
                method,
                &prefix,
                self_type,
                &c_type_name,
                ffi_type,
                is_constructor,
            ) {
                Ok(m) => m,
                Err(e) => {
                    ffi_functions.push(e.to_compile_error());
                    continue;
                }
            };

            if args.generate_header.is_some() {
                headers.push(ffi_method.generate_header());
            }

            let generated = if ffi_method.is_async {
                ffi_method.generate_async()
            } else {
                ffi_method.generate_sync()
            };

            ffi_functions.push(generated);
        }
    }

    if let Some(header_path) = &args.generate_header {
        let mut header_content = String::new();

        header_content.push_str("#pragma once\n\n");
        header_content.push_str("#include <glib.h>\n");
        header_content.push_str("#include <gio/gio.h>\n");
        header_content.push_str("#include <glib-object.h>\n\n");
        header_content.push_str("G_BEGIN_DECLS\n\n");

        match ffi_type {
            types::FfiType::Object | types::FfiType::Shared | types::FfiType::Boxed => {
                header_content.push_str(&format!(
                    "typedef struct _{} {};\n\n",
                    c_type_name_str, c_type_name_str
                ));
            }
            types::FfiType::Enum | types::FfiType::Flags => {}
        }
        header_content.push_str(&format!("GType {}_get_type(void);\n\n", prefix));

        for header in &headers {
            header_content.push_str(header);
            header_content.push('\n');
        }

        header_content.push_str("G_END_DECLS\n");

        if let Err(e) = std::fs::write(header_path.value(), header_content) {
            return syn::Error::new_spanned(
                header_path,
                format!("Failed to write header file: {}", e),
            )
            .to_compile_error()
            .into();
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

    let type_alias = generate_type_alias(ffi_type, self_type, &c_type_name);
    let get_type_fn = generate_get_type_fn(&prefix, self_type);

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
