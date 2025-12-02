// Test that different transfer modes can be used in the same impl block
use glib::subclass::prelude::*;
use gobject_macros::{c_return_type, ffi_impl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "TestMode")]
enum Mode {
    Read,
    Write,
}

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct TestObject;

    #[glib::object_subclass]
    impl ObjectSubclass for TestObject {
        const NAME: &'static str = "TestObject";
        type Type = super::TestObject;
    }

    impl ObjectImpl for TestObject {}
}

glib::wrapper! {
    pub struct TestObject(ObjectSubclass<imp::TestObject>);
}

#[ffi_impl]
impl TestObject {
    // primitive transfer for enum
    fn set_mode(&self, #[c_type(i32, transfer=primitive)] mode: Mode) {
        let _ = mode;
    }

    // none transfer for borrowed string
    fn process_borrowed(
        &self,
        #[c_type(*mut std::os::raw::c_char, transfer=none)] text: String,
    ) -> bool {
        !text.is_empty()
    }

    // full transfer for owned bytes
    fn consume_bytes(
        &self,
        #[c_type(*mut glib::ffi::GBytes, transfer=full)] data: glib::Bytes,
    ) -> i32 {
        data.len() as i32
    }

    // primitive return
    #[c_return_type(i32, transfer=primitive)]
    fn get_mode(&self) -> Mode {
        Mode::Read
    }

    // none return
    #[c_return_type(*mut std::os::raw::c_char, transfer=none)]
    fn get_borrowed_string(&self) -> String {
        "borrowed".to_string()
    }

    // full return
    #[c_return_type(*mut glib::ffi::GBytes, transfer=full)]
    fn create_bytes(&self) -> glib::Bytes {
        glib::Bytes::from_static(b"data")
    }
}

fn main() {}
