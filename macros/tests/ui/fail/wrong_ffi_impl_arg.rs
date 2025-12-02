// Test that ffi_impl requires c_type_name argument (not other names)
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

// Wrong argument name - should be "c_type_name"
#[ffi_impl(type_name = "TestObjectPtr")]
impl TestObject {
    fn test_method(&self) -> bool {
        true
    }
}

fn main() {}
