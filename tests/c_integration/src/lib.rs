use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Calculator;

    #[glib::object_subclass]
    impl ObjectSubclass for Calculator {
        const NAME: &'static str = "TestCalculator";
        type Type = super::Calculator;
    }

    impl ObjectImpl for Calculator {}
}

glib::wrapper! {
    pub struct Calculator(ObjectSubclass<imp::Calculator>);
}

unsafe impl Sync for Calculator {}
unsafe impl Send for Calculator {}

#[ffi_impl(generate_header = "calculator.h")]
impl Calculator {
    fn new() -> Self {
        glib::Object::new()
    }

    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }

    fn is_positive(&self, value: i32) -> bool {
        value > 0
    }

    fn get_message(&self) -> String {
        String::from("Hello from Rust!")
    }

    fn negate(&self, value: i32) -> i32 {
        -value
    }

    fn divide(&self, a: i32, b: i32) -> Result<i32, glib::Error> {
        if b == 0 {
            Err(glib::Error::new(
                glib::FileError::Failed,
                "Division by zero",
            ))
        } else {
            Ok(a / b)
        }
    }

    fn add_optional(&self, a: i32, b: Option<i32>) -> i32 {
        a + b.unwrap_or(0)
    }

    fn compute_sum_and_product(&self, a: i32, b: i32, product: &mut i32) -> i32 {
        *product = a * b;
        a + b
    }

    async fn compute_factorial(&self, n: u32) -> u64 {
        let mut result = 1u64;
        for i in 1..=n {
            result *= i as u64;
        }
        result
    }

    async fn safe_divide(&self, a: i32, b: i32) -> Result<i32, glib::Error> {
        if b == 0 {
            Err(glib::Error::new(
                glib::FileError::Failed,
                "Async division by zero",
            ))
        } else {
            Ok(a / b)
        }
    }
}
