// Test that c_return_type rejects invalid transfer mode
use glib::subclass::prelude::*;
use gobject_macros::{c_return_type, ffi_impl};

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
    // Invalid transfer mode in return type
    #[c_return_type(i32, transfer=borrowed)]
    fn get_value(&self) -> i32 {
        42
    }
}

fn main() {}
