use quote::quote;
use syn::{
    Token, Type,
    parse::{Parse, ParseStream},
};

/// FFI wrapper type category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FfiType {
    #[default]
    Object,
    Boxed,
    Shared,
    Enum,
    Flags,
}

impl FfiType {
    pub(crate) fn is_gobject(&self) -> bool {
        matches!(self, FfiType::Object)
    }

    pub(crate) fn self_c_type(&self) -> Option<syn::Type> {
        match self {
            FfiType::Enum => Some(syn::parse_quote! { i32 }),
            FfiType::Flags => Some(syn::parse_quote! { u32 }),
            FfiType::Object | FfiType::Boxed | FfiType::Shared => None,
        }
    }

    pub(crate) fn self_transfer_mode(&self) -> TransferMode {
        match self {
            FfiType::Enum | FfiType::Flags => TransferMode::Primitive,
            FfiType::Object | FfiType::Boxed | FfiType::Shared => TransferMode::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferMode {
    Primitive,
    None,
    Full,
}

impl TransferMode {
    pub(crate) fn convert_from(
        &self,
        param_name: &syn::Ident,
        ty: &Type,
    ) -> proc_macro2::TokenStream {
        match self {
            TransferMode::Primitive => quote! {
                let #param_name: #ty = unsafe { ::glib::translate::FromGlib::from_glib(#param_name) };
            },
            TransferMode::None => quote! {
                let #param_name: #ty = unsafe { ::glib::translate::FromGlibPtrNone::from_glib_none(#param_name) };
            },
            TransferMode::Full => quote! {
                let #param_name: #ty = unsafe { ::glib::translate::FromGlibPtrFull::from_glib_full(#param_name) };
            },
        }
    }

    pub(crate) fn convert_to(&self, val: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            TransferMode::Primitive => quote! { ::glib::translate::IntoGlib::into_glib(#val) },
            TransferMode::None => quote! { ::glib::translate::ToGlibPtr::to_glib_none(&#val).0 },
            TransferMode::Full => quote! { ::glib::translate::ToGlibPtr::to_glib_full(&#val) },
        }
    }

    pub(crate) fn error_value(&self) -> proc_macro2::TokenStream {
        match self {
            TransferMode::Primitive => quote! { 0 },
            TransferMode::None | TransferMode::Full => quote! { ::std::ptr::null_mut() },
        }
    }
}

#[derive(Clone)]
pub(crate) struct CTypeOverride {
    pub(crate) c_type: Type,
    pub(crate) transfer: TransferMode,
}

impl Parse for CTypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let c_type: Type = input.parse()?;

        if input.parse::<Token![,]>().is_err() {
            return Err(syn::Error::new(
                input.span(),
                "missing required `transfer` parameter (expected: `, transfer=primitive|none|full`)",
            ));
        }

        let transfer_ident: syn::Ident = input.parse()?;
        if transfer_ident != "transfer" {
            return Err(syn::Error::new_spanned(
                transfer_ident,
                "expected `transfer`",
            ));
        }
        input.parse::<Token![=]>()?;

        let mode_ident: syn::Ident = input.parse()?;
        let transfer = match mode_ident.to_string().as_str() {
            "primitive" => TransferMode::Primitive,
            "none" => TransferMode::None,
            "full" => TransferMode::Full,
            _ => {
                return Err(syn::Error::new_spanned(
                    mode_ident,
                    "expected `primitive`, `none`, or `full`",
                ));
            }
        };

        Ok(CTypeOverride { c_type, transfer })
    }
}

pub(crate) struct FfiImplArgs {
    pub(crate) c_type_name: Option<syn::LitStr>,
    pub(crate) prefix: syn::LitStr,
    pub(crate) ty: FfiType,
}

impl FfiImplArgs {
    pub(crate) fn get_c_type_name(&self, rust_type: &syn::Type) -> syn::Result<String> {
        if let Some(ref name) = self.c_type_name {
            return Ok(name.value());
        }

        if let syn::Type::Path(type_path) = rust_type {
            if let Some(last_segment) = type_path.path.segments.last() {
                let type_name = last_segment.ident.to_string();
                return Ok(type_name);
            }
        }

        Err(syn::Error::new_spanned(
            rust_type,
            "Cannot auto-generate c_type_name for this type. Please specify it explicitly with `c_type_name = \"...\"`",
        ))
    }
}

impl Parse for FfiImplArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut c_type_name: Option<syn::LitStr> = None;
        let mut prefix: Option<syn::LitStr> = None;
        let mut ty: Option<FfiType> = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "c_type_name" => {
                    let value: syn::LitStr = input.parse()?;
                    c_type_name = Some(value);
                }
                "prefix" => {
                    let value: syn::LitStr = input.parse()?;
                    prefix = Some(value);
                }
                "ty" => {
                    let ty_value: syn::LitStr = input.parse()?;
                    ty = Some(match ty_value.value().as_str() {
                        "object" => FfiType::Object,
                        "boxed" => FfiType::Boxed,
                        "shared" => FfiType::Shared,
                        "enum" => FfiType::Enum,
                        "flags" => FfiType::Flags,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ty_value,
                                "expected one of: \"object\", \"boxed\", \"shared\", \"enum\", \"flags\"",
                            ));
                        }
                    });
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected `c_type_name`, `prefix`, or `ty`",
                    ));
                }
            }

            // Parse optional comma
            if input.parse::<Token![,]>().is_err() {
                break;
            }
        }

        let prefix = prefix.unwrap_or_else(|| syn::LitStr::new("", proc_macro2::Span::call_site()));

        // Default ty to object if not specified
        let ty = ty.unwrap_or(FfiType::Object);

        Ok(FfiImplArgs {
            c_type_name,
            prefix,
            ty,
        })
    }
}
