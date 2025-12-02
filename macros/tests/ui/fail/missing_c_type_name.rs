// Test that ffi_impl requires c_type_name argument
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

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

// Missing c_type_name argument
#[ffi_impl]
impl TestObject {
    fn test_method(&self) -> bool {
        true
    }
}

fn main() {}
