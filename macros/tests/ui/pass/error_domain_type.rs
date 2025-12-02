// Test FFI generation for glib::ErrorDomain types
use gobject_macros::{ffi_impl, c_return_type};
use glib::{Quark, error::ErrorDomain};

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum, glib::ErrorDomain)]
#[enum_type(name = "MyErrorEnum")]
#[error_domain(name = "MyError")]
pub enum MyError {
    Failed,
    InvalidInput,
    NotFound,
    PermissionDenied,
}

#[ffi_impl(prefix = "my_error", ty = "enum")]
impl MyError {
    #[c_return_type(u32, transfer=primitive)]
    fn quark() -> Quark {
        <MyError as ErrorDomain>::domain()
    }

    #[c_return_type(i32, transfer=primitive)]
    fn from_code(code: i32) -> MyError {
        <MyError as ErrorDomain>::from(code).unwrap_or(MyError::Failed)
    }

    fn is_fatal(&self) -> bool {
        matches!(self, MyError::Failed | MyError::PermissionDenied)
    }

    fn is_recoverable(&self) -> bool {
        matches!(self, MyError::InvalidInput | MyError::NotFound)
    }

    fn message(&self) -> String {
        match self {
            MyError::Failed => "Operation failed".to_string(),
            MyError::InvalidInput => "Invalid input provided".to_string(),
            MyError::NotFound => "Resource not found".to_string(),
            MyError::PermissionDenied => "Permission denied".to_string(),
        }
    }
}

fn main() {}
