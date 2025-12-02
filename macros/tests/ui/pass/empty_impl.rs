// Test that empty impl block is allowed
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

#[ffi_impl(prefix = "test_object")]
impl TestObject {
    // No methods
}

fn main() {}
