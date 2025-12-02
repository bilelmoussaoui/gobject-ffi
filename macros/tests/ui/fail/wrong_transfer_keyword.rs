// Test that wrong keyword is rejected (e.g., "mode" instead of "transfer")
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

#[ffi_impl]
impl TestObject {
    // Wrong keyword - should be "transfer" not "mode"
    fn test_method(
        &self,
        #[c_type(i32, mode=primitive)] value: i32,
    ) -> bool {
        value > 0
    }
}

fn main() {}
