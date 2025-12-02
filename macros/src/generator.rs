use quote::quote;

use crate::{context::FfiMethodContext, params::FfiComponents, types::TransferMode};

pub enum FfiKind {
    Constructor,
    Method {
        source_for_task: proc_macro2::TokenStream,
    },
}

#[derive(Clone)]
pub struct AsyncFunctionNames {
    pub async_name: syn::Ident,
    pub finish_name: syn::Ident,
    pub sync_name: syn::Ident,
}

impl AsyncFunctionNames {
    pub fn from_prefix(ffi_prefix: &str, span: proc_macro2::Span) -> Self {
        Self {
            async_name: syn::Ident::new(ffi_prefix, span),
            finish_name: syn::Ident::new(&format!("{}_finish", ffi_prefix), span),
            sync_name: syn::Ident::new(&format!("{}_sync", ffi_prefix), span),
        }
    }
}

pub struct AsyncFfiParams<'a> {
    pub c_type_name: &'a syn::Ident,
    pub ok_type: &'a proc_macro2::TokenStream,
    pub sync_call_args: proc_macro2::TokenStream,
    pub kind: FfiKind,
    pub ffi_type: crate::types::FfiType,
}

pub struct FfiBuilder {
    ctx: FfiMethodContext,
    components: FfiComponents,
    body: proc_macro2::TokenStream,
    params_override: Option<proc_macro2::TokenStream>,
    conversions_override: Option<Vec<proc_macro2::TokenStream>>,
}

impl FfiBuilder {
    pub fn from_context(ctx: FfiMethodContext, components: FfiComponents) -> Self {
        Self {
            ctx,
            components,
            body: quote! {},
            params_override: None,
            conversions_override: None,
        }
    }

    pub fn with_body(mut self, body: proc_macro2::TokenStream) -> Self {
        self.body = body;
        self
    }

    pub fn with_params(mut self, params: proc_macro2::TokenStream) -> Self {
        self.params_override = Some(params);
        self
    }

    pub fn with_args(mut self, args: Vec<proc_macro2::TokenStream>) -> Self {
        self.conversions_override = Some(args);
        self
    }

    pub fn build_sync(self) -> proc_macro2::TokenStream {
        let fn_name = syn::Ident::new(&self.ctx.ffi_prefix, self.ctx.fn_name.span());
        let params = self.params_override.unwrap_or(self.components.params);
        let conversions = self
            .conversions_override
            .unwrap_or(self.components.conversions);
        let body = self.body;
        let return_type = &self.components.return_type;
        let ok_handler = &self.components.ok_handler;
        let err_handler = &self.components.err_handler;
        let is_fallible = self.ctx.is_fallible;

        let error_param = if is_fallible {
            quote! { error: *mut *mut ::glib::ffi::GError, }
        } else {
            quote! {}
        };

        let ok_handler_str = ok_handler.to_string();
        let is_void = ok_handler_str.trim().is_empty();

        let function_body = if is_fallible {
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
        } else if is_void {
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

    pub fn build_async(self, async_params: AsyncFfiParams) -> proc_macro2::TokenStream {
        let function_names = self
            .ctx
            .async_names
            .as_ref()
            .expect("async_names must be Some when building async FFI");
        let async_fn_name = &function_names.async_name;
        let finish_fn_name = &function_names.finish_name;
        let sync_fn_name = &function_names.sync_name;
        let c_type_name = async_params.c_type_name;
        let ok_type = async_params.ok_type;
        let sync_call_args = async_params.sync_call_args;
        let kind = async_params.kind;
        let ffi_type = async_params.ffi_type;
        let params = self.params_override.unwrap_or(self.components.params);
        let conversions = self
            .conversions_override
            .unwrap_or(self.components.conversions);
        let body = self.body;
        let return_type = &self.components.return_type;
        let ok_handler = &self.components.ok_handler;
        let err_handler = &self.components.err_handler;
        let is_fallible = self.ctx.is_fallible;

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

        let is_void = crate::utils::is_unit_type_token(ok_type);
        let task_type = if is_void {
            quote! { bool }
        } else {
            ok_type.clone()
        };

        let (source_object_for_task, callback_source_expr) = if ffi_type.is_gobject() {
            match &kind {
                FfiKind::Constructor => {
                    let obj_conversion = TransferMode::None.convert_to(
                        quote! { ::glib::object::Cast::upcast_ref::<::glib::Object>(obj) },
                    );
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
                }
                FfiKind::Method { source_for_task } => {
                    let source_conversion = TransferMode::None
                        .convert_to(quote! { ::glib::object::Cast::upcast_ref::<::glib::Object>(&#source_for_task) });
                    (
                        quote! { Some(&#source_for_task) },
                        quote! {
                            let callback_source = #source_conversion;
                        },
                    )
                }
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

        let finish_self_param = match &kind {
            FfiKind::Constructor => quote! {},
            FfiKind::Method { .. } => {
                // For enum/flags, use the primitive type; for others use pointer
                if let Some(self_c_ty) = ffi_type.self_c_type() {
                    quote! { _self: #self_c_ty, }
                } else {
                    quote! { _self: *mut #c_type_name, }
                }
            }
        };

        let sync_self_param = match &kind {
            FfiKind::Constructor => quote! {},
            FfiKind::Method { .. } => quote! { self_param, },
        };

        let error_param = if is_fallible {
            quote! { error: *mut *mut ::glib::ffi::GError, }
        } else {
            quote! {}
        };

        let sync_error_arg = if is_fallible {
            quote! { , error }
        } else {
            quote! {}
        };

        let task_result_expr = if is_fallible {
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

        let finish_body = if is_fallible {
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
