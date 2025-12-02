#![doc = include_str!("../../README.md")]

use std::{os::raw::c_char, path::PathBuf};

use glib::translate::*;
pub use gobject_macros::{c_return_type, ffi_impl};

/// Trait for types that can be converted to/from C FFI representations
///
/// # Example
///
/// Implementing for a custom wrapper type:
///
/// ```ignore
/// use gobject_ffi::FfiConvert;
/// use glib::translate::*;
/// use std::os::raw::c_char;
///
/// // Custom type that wraps a GObject pointer
/// pub struct MyObject(*mut glib::gobject_ffi::GObject);
///
/// impl FfiConvert for MyObject {
///     type CType = *mut glib::gobject_ffi::GObject;
///
///     unsafe fn from_c_borrowed(value: Self::CType) -> Self {
///         // Increment reference count for borrowed pointer
///         unsafe {
///             glib::gobject_ffi::g_object_ref(value as *mut _);
///         }
///         MyObject(value)
///     }
///
///     fn to_c_owned(self) -> Self::CType {
///         // Transfer ownership without incrementing refcount
///         let ptr = self.0;
///         std::mem::forget(self);
///         ptr
///     }
///
///     fn c_error_value() -> Self::CType {
///         std::ptr::null_mut()
///     }
/// }
/// ```
pub trait FfiConvert: Sized {
    /// The C type representation
    type CType: Copy;

    /// Convert from a borrowed C value to Rust type
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` is a valid representation of
    /// `Self::CType`
    unsafe fn from_c_borrowed(value: Self::CType) -> Self;

    /// Transfer ownership to C as return value
    fn to_c_owned(self) -> Self::CType;

    /// Error value returned when Result<T, E> fails
    fn c_error_value() -> Self::CType;
}

impl FfiConvert for bool {
    type CType = glib::ffi::gboolean;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        IntoGlib::into_glib(self)
    }

    fn c_error_value() -> Self::CType {
        glib::ffi::GFALSE
    }
}

impl FfiConvert for String {
    type CType = *mut c_char;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        ToGlibPtr::to_glib_full(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl FfiConvert for PathBuf {
    type CType = *mut c_char;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        ToGlibPtr::to_glib_full(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl FfiConvert for Vec<u8> {
    type CType = *mut glib::ffi::GBytes;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        let bytes: glib::Bytes = unsafe { from_glib_none(value) };
        bytes.to_vec()
    }

    fn to_c_owned(self) -> Self::CType {
        let bytes = glib::Bytes::from_owned(self);
        IntoGlibPtr::into_glib_ptr(bytes)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

macro_rules! impl_ffi_convert_for_primitive {
    ($rust_type:ty, $c_type:ty, $error_val:expr) => {
        impl FfiConvert for $rust_type {
            type CType = $c_type;

            unsafe fn from_c_borrowed(value: Self::CType) -> Self {
                value as Self
            }

            fn to_c_owned(self) -> Self::CType {
                self as Self::CType
            }

            fn c_error_value() -> Self::CType {
                $error_val
            }
        }
    };
}

impl_ffi_convert_for_primitive!(i8, i8, -1);
impl_ffi_convert_for_primitive!(u8, u8, 0);
impl_ffi_convert_for_primitive!(i16, i16, -1);
impl_ffi_convert_for_primitive!(u16, u16, 0);
impl_ffi_convert_for_primitive!(i32, i32, -1);
impl_ffi_convert_for_primitive!(u32, u32, 0);
impl_ffi_convert_for_primitive!(i64, i64, -1);
impl_ffi_convert_for_primitive!(u64, u64, 0);
impl_ffi_convert_for_primitive!(f32, f32, 0.0);
impl_ffi_convert_for_primitive!(f64, f64, 0.0);

impl FfiConvert for () {
    type CType = ();

    unsafe fn from_c_borrowed(_value: Self::CType) -> Self {}

    fn to_c_owned(self) -> Self::CType {}

    fn c_error_value() -> Self::CType {}
}

impl FfiConvert for glib::Bytes {
    type CType = *mut glib::ffi::GBytes;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        ToGlibPtr::to_glib_full(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl FfiConvert for glib::Variant {
    type CType = *mut glib::ffi::GVariant;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        ToGlibPtr::to_glib_full(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl FfiConvert for glib::GString {
    type CType = *mut c_char;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        ToGlibPtr::to_glib_full(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl FfiConvert for Vec<String> {
    type CType = *mut glib::ffi::GList;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        unsafe { FromGlibPtrContainer::from_glib_none(value) }
    }

    fn to_c_owned(self) -> Self::CType {
        glib::translate::ToGlibContainerFromSlice::to_glib_full_from_slice(&self)
    }

    fn c_error_value() -> Self::CType {
        std::ptr::null_mut()
    }
}

impl<T: FfiConvert> FfiConvert for Option<T>
where
    T::CType: PartialEq,
{
    type CType = T::CType;

    unsafe fn from_c_borrowed(value: Self::CType) -> Self {
        if value == T::c_error_value() {
            None
        } else {
            Some(unsafe { T::from_c_borrowed(value) })
        }
    }

    fn to_c_owned(self) -> Self::CType {
        match self {
            Some(val) => T::to_c_owned(val),
            None => T::c_error_value(),
        }
    }

    fn c_error_value() -> Self::CType {
        T::c_error_value()
    }
}
