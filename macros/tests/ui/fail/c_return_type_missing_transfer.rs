// Test that c_return_type requires transfer parameter
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
    // Missing transfer parameter
    #[c_return_type(i32)]
    fn get_value(&self) -> i32 {
        42
    }
}

fn main() {}
