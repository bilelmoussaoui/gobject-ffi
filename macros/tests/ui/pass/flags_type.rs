// Test FFI generation for glib::Flags types
use gobject_macros::{ffi_impl, c_return_type};

#[glib::flags(name = "MyPermissions")]
pub enum Permissions {
    READ = 0b0001,
    WRITE = 0b0010,
    EXECUTE = 0b0100,
    DELETE = 0b1000,
}

#[ffi_impl(prefix = "my", ty = "flags")]
impl Permissions {
    #[c_return_type(u32, transfer=primitive)]
    fn none() -> Permissions {
        Permissions::empty()
    }

    #[c_return_type(u32, transfer=primitive)]
    fn read_only() -> Permissions {
        Permissions::READ
    }

    #[c_return_type(u32, transfer=primitive)]
    fn read_write() -> Permissions {
        Permissions::READ | Permissions::WRITE
    }

    fn can_read(&self) -> bool {
        self.contains(Permissions::READ)
    }

    fn can_write(&self) -> bool {
        self.contains(Permissions::WRITE)
    }

    fn can_execute(&self) -> bool {
        self.contains(Permissions::EXECUTE)
    }

    #[c_return_type(u32, transfer=primitive)]
    fn with(&self, #[c_type(u32, transfer=primitive)] other: Permissions) -> Permissions {
        *self | other
    }

    fn has_all(&self, #[c_type(u32, transfer=primitive)] other: Permissions) -> bool {
        self.contains(other)
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    // Test async method
    async fn validate_permissions(&self) -> Result<bool, glib::Error> {
        if self.is_empty() {
            Err(glib::Error::new(glib::FileError::Failed, "No permissions set"))
        } else {
            Ok(true)
        }
    }

    // Test fallible sync method
    #[c_return_type(u32, transfer=primitive)]
    fn add_permission(&self, #[c_type(u32, transfer=primitive)] perm: Permissions) -> Result<Permissions, glib::Error> {
        if self.contains(perm) {
            Err(glib::Error::new(glib::FileError::Failed, "Permission already set"))
        } else {
            Ok(*self | perm)
        }
    }
}

fn main() {
    #[allow(unused_imports)]
    use ffi::MyPermissions;
}
