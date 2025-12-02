// Test handling of many parameters
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

#[ffi_impl(prefix = "calculator")]
impl Calculator {
    // Many parameters
    fn compute(
        &self,
        a: i32,
        b: i32,
        c: i32,
        d: i32,
        e: i32,
        f: String,
        g: bool,
        h: Option<String>,
    ) -> i32 {
        let _ = (f, g, h);
        a + b + c + d + e
    }

    // Many parameters async
    async fn async_compute(
        &self,
        a: i32,
        b: String,
        c: bool,
        d: Option<i32>,
        e: String,
    ) -> Result<String, glib::Error> {
        let _ = (b, c, d, e);
        Ok(a.to_string())
    }
}

fn main() {}
