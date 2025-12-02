// Test to verify transfer mode annotations compile correctly
use glib::subclass::prelude::*;
use gobject_macros::{c_return_type, ffi_impl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "TestStatus")]
enum Status {
    Idle,
    Running,
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
    // Test transfer=primitive for enums
    fn check_status_primitive(
        &self,
        #[c_type(i32, transfer=primitive)] status: Status,
    ) -> bool {
        status == Status::Running
    }

    // Test transfer=primitive for external enum
    fn check_bus_type(
        &self,
        #[c_type(i32, transfer=primitive)] bus_type: gio::BusType,
    ) -> bool {
        !matches!(bus_type, gio::BusType::None)
    }

    // Test primitive types (using FfiConvert)
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    // Test String (using FfiConvert)
    fn process_string(&self, input: String) -> String {
        input.to_uppercase()
    }

    // Test transfer=none for borrowed string (C doesn't take ownership)
    fn check_string_none(
        &self,
        #[c_type(*mut std::os::raw::c_char, transfer=none)] text: String,
    ) -> bool {
        !text.is_empty()
    }

    // Test transfer=full for owned string (C takes ownership)
    fn check_string_full(
        &self,
        #[c_type(*mut std::os::raw::c_char, transfer=full)] text: String,
    ) -> bool {
        text.len() > 5
    }

    // Test transfer=none for borrowed bytes
    fn check_bytes_none(
        &self,
        #[c_type(*mut glib::ffi::GBytes, transfer=none)] data: glib::Bytes,
    ) -> bool {
        !data.is_empty()
    }

    // Test transfer=full for owned bytes
    fn check_bytes_full(
        &self,
        #[c_type(*mut glib::ffi::GBytes, transfer=full)] data: glib::Bytes,
    ) -> bool {
        data.len() > 10
    }

    // Test return with transfer=none (C borrows, doesn't own)
    #[c_return_type(*mut std::os::raw::c_char, transfer=none)]
    fn get_borrowed_string(&self) -> String {
        "borrowed".to_string()
    }

    // Test return with transfer=full (C owns the returned value)
    #[c_return_type(*mut std::os::raw::c_char, transfer=full)]
    fn get_owned_string(&self) -> String {
        "owned".to_string()
    }
}

fn main() {}
