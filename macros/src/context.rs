use syn::{ImplItemFn, ReturnType};

use crate::generator::AsyncFunctionNames;

pub(crate) struct FfiMethodContext {
    pub(crate) fn_name: syn::Ident,
    pub(crate) ffi_prefix: String,
    pub(crate) is_async: bool,
    pub(crate) is_fallible: bool,
    pub(crate) async_names: Option<AsyncFunctionNames>,
}

impl FfiMethodContext {
    pub(crate) fn new(method: &ImplItemFn, prefix: &str) -> Self {
        let fn_name = method.sig.ident.clone();
        let ffi_prefix = format!("{}_{}", prefix, fn_name);
        let is_async = method.sig.asyncness.is_some();
        let is_fallible = check_fallibility(&method.sig.output);

        let async_names = if is_async {
            Some(AsyncFunctionNames::from_prefix(&ffi_prefix, fn_name.span()))
        } else {
            None
        };

        Self {
            fn_name,
            ffi_prefix,
            is_async,
            is_fallible,
            async_names,
        }
    }
}

fn check_fallibility(return_type: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = return_type {
        crate::utils::is_result_type(ty)
    } else {
        false
    }
}
