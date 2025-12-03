use quote::quote;
use syn::Type;

use crate::types::{FfiType, TransferMode};

struct AsyncFunctionNames {
    async_name: syn::Ident,
    finish_name: syn::Ident,
    sync_name: syn::Ident,
}

impl AsyncFunctionNames {
    fn from_prefix(ffi_prefix: &str, span: proc_macro2::Span) -> Self {
        Self {
            async_name: syn::Ident::new(ffi_prefix, span),
            finish_name: syn::Ident::new(&format!("{}_finish", ffi_prefix), span),
            sync_name: syn::Ident::new(&format!("{}_sync", ffi_prefix), span),
        }
    }
}

struct FfiParam {
    name: syn::Ident,
    rust_type: Type,
    c_type: proc_macro2::TokenStream,
    c_type_override: Option<crate::types::CTypeOverride>,
}

impl FfiParam {
    fn extract_from_method(method: &syn::ImplItemFn, skip_self: bool) -> syn::Result<Vec<Self>> {
        use syn::{FnArg, Pat};

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
                        let rust_type = (*pat_type.ty).clone();
                        let c_type_override = crate::utils::extract_c_type(&pat_type.attrs)?;

                        let c_type = if let Some(ref override_) = c_type_override {
                            let c = &override_.c_type;
                            quote! { #c }
                        } else {
                            crate::utils::rust_type_to_c_type(&rust_type)
                        };

                        params.push(Self {
                            name: param_name.clone(),
                            rust_type,
                            c_type,
                            c_type_override,
                        });
                    }
                }
            }
        }
        Ok(params)
    }

    fn generate_conversion(&self) -> proc_macro2::TokenStream {
        let param_name = &self.name;
        let ty = &self.rust_type;

        if crate::utils::is_mutable_reference(ty) {
            return quote! {
                let #param_name: #ty = unsafe { &mut *#param_name };
            };
        }

        if let Some(ref override_) = self.c_type_override {
            override_.transfer.convert_from(param_name, ty)
        } else {
            quote! {
                let #param_name: #ty = unsafe { <#ty as ::gobject_ffi::FfiConvert>::from_c_borrowed(#param_name) };
            }
        }
    }
}

struct FfiReturn {
    rust_type: Type,
    c_type: proc_macro2::TokenStream,
    transfer: TransferMode,
}

impl FfiReturn {
    fn new(
        rust_type: Type,
        c_return_type_override: Option<crate::types::CTypeOverride>,
        ffi_type: FfiType,
        c_type_name: &syn::Ident,
        is_constructor: bool,
    ) -> Self {
        let (c_type, transfer) = if let Some(ref override_) = c_return_type_override {
            let c = &override_.c_type;
            (quote! { #c }, override_.transfer)
        } else if crate::utils::is_unit_type(&rust_type) {
            (quote! { () }, TransferMode::None)
        } else if is_constructor
            && (ffi_type.is_gobject() || matches!(ffi_type, FfiType::Boxed | FfiType::Shared))
        {
            // For constructors of GObject, Boxed, and Shared types, use pointer and
            // transfer mode
            (quote! { *mut #c_type_name }, TransferMode::Full)
        } else {
            // For methods returning basic types or other types, use FfiConvert
            let c_type = if let Some(inner_type) = crate::utils::extract_option_inner(&rust_type) {
                quote! { <#inner_type as ::gobject_ffi::FfiConvert>::CType }
            } else {
                quote! { <#rust_type as ::gobject_ffi::FfiConvert>::CType }
            };
            (c_type, TransferMode::Full)
        };

        Self {
            rust_type,
            c_type,
            transfer,
        }
    }

    fn is_void(&self) -> bool {
        crate::utils::is_unit_type(&self.rust_type)
    }

    fn generate_ok_handler(&self) -> proc_macro2::TokenStream {
        if self.is_void() {
            return quote! { () };
        }

        if self.uses_ffi_convert() {
            let rust_type = &self.rust_type;
            quote! { <#rust_type as ::gobject_ffi::FfiConvert>::to_c_owned(val) }
        } else {
            self.transfer.convert_to(quote! { val })
        }
    }

    fn generate_err_handler(&self) -> proc_macro2::TokenStream {
        if self.is_void() {
            return quote! { () };
        }

        if self.uses_ffi_convert() {
            let rust_type = &self.rust_type;
            quote! { <#rust_type as ::gobject_ffi::FfiConvert>::c_error_value() }
        } else {
            self.transfer.error_value()
        }
    }

    fn uses_ffi_convert(&self) -> bool {
        // Check if c_type contains "FfiConvert" to determine if we're using the trait
        self.c_type.to_string().contains("FfiConvert")
    }
}

pub(crate) struct FfiMethod {
    rust_name: syn::Ident,
    self_type: Option<Type>,
    c_type_name: syn::Ident,
    ffi_prefix: String,
    params: Vec<FfiParam>,
    return_info: FfiReturn,
    pub(crate) is_async: bool,
    is_fallible: bool,
    async_names: Option<AsyncFunctionNames>,
    ffi_type: FfiType,
}

impl FfiMethod {
    pub(crate) fn from_method(
        method: &syn::ImplItemFn,
        prefix: &str,
        impl_self_type: &Type,
        c_type_name: &syn::Ident,
        ffi_type: FfiType,
        is_constructor: bool,
    ) -> syn::Result<Self> {
        use syn::ReturnType;

        let fn_name = method.sig.ident.clone();
        let ffi_prefix = format!("{}_{}", prefix, fn_name);
        let is_async = method.sig.asyncness.is_some();
        let is_fallible = crate::utils::check_fallibility(&method.sig.output);

        let async_names = if is_async {
            Some(AsyncFunctionNames::from_prefix(&ffi_prefix, fn_name.span()))
        } else {
            None
        };

        let params = FfiParam::extract_from_method(method, !is_constructor)?;

        let c_return_type = crate::utils::extract_c_return_type(&method.attrs)?;

        let rust_return_type = if is_constructor {
            syn::parse_quote! { super::#impl_self_type }
        } else {
            match &method.sig.output {
                ReturnType::Default => syn::parse_quote! { () },
                ReturnType::Type(_, ty) => {
                    if is_fallible {
                        crate::utils::extract_result_ok_type_as_type(ty)
                    } else {
                        (**ty).clone()
                    }
                }
            }
        };

        let return_info = FfiReturn::new(
            rust_return_type,
            c_return_type,
            ffi_type,
            c_type_name,
            is_constructor,
        );

        let method_self_type = if is_constructor {
            None
        } else {
            Some(syn::parse_quote! { super::#impl_self_type })
        };

        Ok(Self {
            rust_name: fn_name,
            self_type: method_self_type,
            c_type_name: c_type_name.clone(),
            ffi_prefix,
            params,
            return_info,
            is_async,
            is_fallible,
            async_names,
            ffi_type,
        })
    }

    fn is_constructor(&self) -> bool {
        self.self_type.is_none()
    }

    fn param_names(&self) -> Vec<&syn::Ident> {
        self.params.iter().map(|p| &p.name).collect()
    }

    fn generate_ffi_params_inner(&self) -> proc_macro2::TokenStream {
        let params = self.params.iter().map(|p| {
            let name = &p.name;
            let c_type = &p.c_type;
            quote! { #name: #c_type, }
        });
        quote! { #(#params)* }
    }

    fn generate_conversions_inner(&self) -> Vec<proc_macro2::TokenStream> {
        self.params
            .iter()
            .map(|p| p.generate_conversion())
            .collect()
    }

    fn generate_sync_call_args(&self) -> proc_macro2::TokenStream {
        let param_args = if self.params.is_empty() {
            quote! {}
        } else {
            let param_names = self.param_names();
            quote! { #(#param_names,)* }
        };

        // For methods, prepend self_param
        if self.is_constructor() {
            param_args
        } else {
            quote! { self_param, #param_args }
        }
    }

    fn generate_param_call_args(&self) -> proc_macro2::TokenStream {
        if self.params.is_empty() {
            quote! {}
        } else {
            let param_names = self.param_names();
            quote! { #(#param_names),* }
        }
    }

    fn generate_body(&self) -> proc_macro2::TokenStream {
        let fn_name = &self.rust_name;
        let param_call_args = self.generate_param_call_args();

        if self.is_constructor() {
            let return_type = &self.return_info.rust_type;
            quote! { #return_type::#fn_name(#param_call_args) }
        } else {
            quote! { self_param.#fn_name(#param_call_args) }
        }
    }

    fn generate_self_ffi_type(&self) -> Option<proc_macro2::TokenStream> {
        self.self_type.as_ref()?;

        if let Some(c_type) = self.ffi_type.self_c_type() {
            Some(quote! { #c_type })
        } else {
            let c_type_name = &self.c_type_name;
            Some(quote! { *mut #c_type_name })
        }
    }

    fn generate_self_conversion(&self) -> Option<proc_macro2::TokenStream> {
        let self_type = self.self_type.as_ref()?;

        let self_param_ident = syn::Ident::new("self_param", proc_macro2::Span::call_site());
        let transfer = if self.ffi_type.self_c_type().is_some() {
            self.ffi_type.self_transfer_mode()
        } else {
            TransferMode::None
        };

        Some(transfer.convert_from(&self_param_ident, self_type))
    }

    fn generate_ffi_params(&self) -> proc_macro2::TokenStream {
        if let Some(self_ffi_type) = self.generate_self_ffi_type() {
            let ffi_params = self.generate_ffi_params_inner();
            quote! {
                self_param: #self_ffi_type,
                #ffi_params
            }
        } else {
            self.generate_ffi_params_inner()
        }
    }

    fn generate_conversions(&self) -> Vec<proc_macro2::TokenStream> {
        let mut conversions = Vec::new();

        if let Some(self_conversion) = self.generate_self_conversion() {
            conversions.push(self_conversion);
        }

        conversions.extend(self.generate_conversions_inner());

        // For async methods on GObject types, clone self_param for use as Task source
        if self.is_async && self.self_type.is_some() && self.ffi_type.is_gobject() {
            conversions.push(quote! {
                let source_for_task = self_param.clone();
            });
        }

        conversions
    }

    pub(crate) fn generate_sync(self) -> proc_macro2::TokenStream {
        let fn_name = syn::Ident::new(&self.ffi_prefix, self.rust_name.span());
        let params = self.generate_ffi_params();
        let conversions = self.generate_conversions();
        let body = self.generate_body();
        let return_type = &self.return_info.c_type;
        let ok_handler = self.return_info.generate_ok_handler();
        let err_handler = self.return_info.generate_err_handler();

        let error_param = if self.is_fallible {
            quote! { error: *mut *mut ::glib::ffi::GError, }
        } else {
            quote! {}
        };

        let function_body = if self.is_fallible {
            quote! {
                match #body {
                    Ok(val) => #ok_handler,
                    Err(e) => {
                        if !error.is_null() {
                            unsafe {
                                *error = ::glib::translate::IntoGlibPtr::into_glib_ptr(e);
                            }
                        }
                        #err_handler
                    }
                }
            }
        } else if self.return_info.is_void() {
            quote! {
                #body;
            }
        } else {
            quote! {
                let val = #body;
                #ok_handler
            }
        };

        quote! {
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #fn_name(
                #params
                #error_param
            ) -> #return_type {
                #(#conversions)*

                #function_body
            }
        }
    }

    pub(crate) fn generate_async(self) -> proc_macro2::TokenStream {
        let function_names = self
            .async_names
            .as_ref()
            .expect("async_names must be Some when building async FFI");
        let async_fn_name = &function_names.async_name;
        let finish_fn_name = &function_names.finish_name;
        let sync_fn_name = &function_names.sync_name;
        let c_type_name = &self.c_type_name;
        let ffi_type = self.ffi_type;

        let sync_call_args = self.generate_sync_call_args();
        let params = self.generate_ffi_params();
        let conversions = self.generate_conversions();
        let body = self.generate_body();
        let return_type = &self.return_info.c_type;
        let ok_handler = self.return_info.generate_ok_handler();
        let err_handler = self.return_info.generate_err_handler();

        let cancellable_ident = syn::Ident::new("cancellable", proc_macro2::Span::call_site());
        let cancellable_type: syn::Type =
            syn::parse_quote! { ::std::option::Option<::gio::Cancellable> };
        let cancellable_conversion =
            TransferMode::None.convert_from(&cancellable_ident, &cancellable_type);

        let result_ident = syn::Ident::new("result", proc_macro2::Span::call_site());
        let async_result_type: syn::Type = syn::parse_quote! { ::gio::AsyncResult };
        let result_conversion = TransferMode::None.convert_from(&result_ident, &async_result_type);

        let async_result_to_c = TransferMode::None.convert_to(quote! { async_result });
        let callback_result_to_c = TransferMode::None
            .convert_to(quote! { callback_data.result.expect("callback was not called") });

        let is_void = self.return_info.is_void();
        let task_type = if is_void {
            quote! { bool }
        } else {
            let ok_type = &self.return_info.rust_type;
            quote! { #ok_type }
        };

        let (source_object_for_task, callback_source_expr) = if ffi_type.is_gobject() {
            if self.is_constructor() {
                let obj_conversion = TransferMode::None
                    .convert_to(quote! { ::glib::object::Cast::upcast_ref::<::glib::Object>(obj) });
                (
                    quote! { None::<&::gio::Cancellable> },
                    quote! {
                        let callback_source = task_result
                            .as_ref()
                            .ok()
                            .map(|obj| #obj_conversion)
                            .unwrap_or(::std::ptr::null_mut());
                    },
                )
            } else {
                let source_conversion = TransferMode::None.convert_to(
                    quote! { ::glib::object::Cast::upcast_ref::<::glib::Object>(&source_for_task) },
                );
                (
                    quote! { Some(&source_for_task) },
                    quote! {
                        let callback_source = #source_conversion;
                    },
                )
            }
        } else {
            // For non-GObject types, always use None as source and null_mut for callback
            (
                quote! { None::<&::gio::Cancellable> },
                quote! {
                    let callback_source = ::std::ptr::null_mut();
                },
            )
        };

        let finish_self_param = if self.is_constructor() {
            quote! {}
        } else {
            // For enum/flags, use the primitive type; for others use pointer
            if let Some(self_c_ty) = ffi_type.self_c_type() {
                quote! { _self: #self_c_ty, }
            } else {
                quote! { _self: *mut #c_type_name, }
            }
        };

        let sync_self_param = if self.is_constructor() {
            quote! {}
        } else {
            quote! { self_param, }
        };

        let error_param = if self.is_fallible {
            quote! { error: *mut *mut ::glib::ffi::GError, }
        } else {
            quote! {}
        };

        let sync_error_arg = if self.is_fallible {
            quote! { , error }
        } else {
            quote! {}
        };

        let task_result_expr = if self.is_fallible {
            if is_void {
                quote! { result.map(|_| true) }
            } else {
                quote! { result }
            }
        } else if is_void {
            quote! { { let _ = result; Ok(true) } }
        } else {
            quote! { Ok(result) }
        };

        let finish_body = if self.is_fallible {
            quote! {
                match ::glib::object::Cast::downcast::<::gio::Task<#task_type>>(result) {
                    Ok(task) => match unsafe { task.propagate() } {
                        Ok(val) => #ok_handler,
                        Err(e) => {
                            if !error.is_null() {
                                unsafe {
                                    *error = ::glib::translate::IntoGlibPtr::into_glib_ptr(e);
                                }
                            }
                            #err_handler
                        }
                    },
                    Err(_) => #err_handler,
                }
            }
        } else if is_void {
            quote! {
                match ::glib::object::Cast::downcast::<::gio::Task<#task_type>>(result) {
                    Ok(task) => {
                        let _ = unsafe { task.propagate() }.unwrap();
                        #ok_handler
                    }
                    Err(_) => #err_handler,
                }
            }
        } else {
            quote! {
                match ::glib::object::Cast::downcast::<::gio::Task<#task_type>>(result) {
                    Ok(task) => {
                        let val = unsafe { task.propagate() }.unwrap();
                        #ok_handler
                    }
                    Err(_) => #err_handler,
                }
            }
        };

        quote! {
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #async_fn_name(
                #params
                cancellable: *mut ::gio::ffi::GCancellable,
                callback: ::gio::ffi::GAsyncReadyCallback,
                user_data: ::glib::ffi::gpointer,
            ) {
                #(#conversions)*

                #cancellable_conversion

                ::glib::MainContext::default().spawn_local(async move {
                    let task_result = if let Some(ref cancellable) = cancellable {
                        let (abortable_op, abort_handle) = ::futures_util::future::abortable(
                            #body
                        );

                        let handler_id = ::gio::prelude::CancellableExtManual::connect_cancelled(cancellable, move |_| {
                            abort_handle.abort();
                        });

                        let abortable_result = abortable_op.await;

                        if let Some(id) = handler_id {
                            ::gio::prelude::CancellableExtManual::disconnect_cancelled(cancellable, id);
                        }

                        match abortable_result {
                            Ok(result) => #task_result_expr,
                            Err(_aborted) => Err(::glib::Error::new(
                                ::gio::IOErrorEnum::Cancelled,
                                "Operation was cancelled",
                            )),
                        }
                    } else {
                        let result = #body.await;
                        #task_result_expr
                    };

                    let task = unsafe {
                        ::gio::Task::new(
                            #source_object_for_task,
                            cancellable.as_ref(),
                            |_task, _result| {},
                        )
                    };

                    let async_result = ::glib::object::Cast::upcast_ref::<::gio::AsyncResult>(&task).clone();

                    #callback_source_expr

                    unsafe {
                        task.return_result(task_result);
                    }

                    if let Some(cb) = callback {
                        unsafe {
                            cb(callback_source, #async_result_to_c, user_data);
                        }
                    }
                });
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #finish_fn_name(
                #finish_self_param
                result: *mut ::gio::ffi::GAsyncResult,
                #error_param
            ) -> #return_type {
                #result_conversion

                #finish_body
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #sync_fn_name(
                #params
                cancellable: *mut ::gio::ffi::GCancellable,
                #error_param
            ) -> #return_type {
                struct CallbackData {
                    result: ::std::option::Option<::gio::AsyncResult>,
                    loop_: ::glib::MainLoop,
                }

                unsafe extern "C" fn callback(
                    _source: *mut ::glib::gobject_ffi::GObject,
                    result: *mut ::gio::ffi::GAsyncResult,
                    user_data: ::glib::ffi::gpointer,
                ) {
                    let data = &mut *(user_data as *mut CallbackData);
                    #result_conversion
                    data.result = Some(result);
                    data.loop_.quit();
                }

                let context = ::glib::MainContext::default();
                let loop_ = ::glib::MainLoop::new(Some(&context), false);

                let mut callback_data = CallbackData {
                    result: None,
                    loop_: loop_.clone(),
                };

                unsafe {
                    #async_fn_name(
                        #sync_call_args
                        cancellable,
                        Some(callback),
                        &mut callback_data as *mut _ as ::glib::ffi::gpointer,
                    );
                }

                loop_.run();

                let result = #callback_result_to_c;
                unsafe { #finish_fn_name(#sync_self_param result #sync_error_arg) }
            }
        }
    }
}
