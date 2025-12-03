// Test basic synchronous method generation
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct Calculator;

    #[glib::object_subclass]
    impl ObjectSubclass for Calculator {
        const NAME: &'static str = "Calculator";
        type Type = super::Calculator;
    }

    impl ObjectImpl for Calculator {}
}

glib::wrapper! {
    pub struct Calculator(ObjectSubclass<imp::Calculator>);
}

#[ffi_impl]
impl Calculator {
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    fn negate(&self, value: i32) -> i32 {
        -value
    }

    fn is_positive(&self, value: i32) -> bool {
        value > 0
    }
}

fn main() {
    #[allow(unused_imports)]
    use ffi::Calculator;
}
