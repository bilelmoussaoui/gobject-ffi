# GObject FFI Macros

Rust procedural macros for automatically generating C FFI wrappers for GObject-based types.

## Overview

This workspace provides the `#[ffi_impl]` macro that transforms Rust methods into C FFI functions following GObject conventions, enabling seamless language interoperability via GObject introspection.

## Minimal Example

```rust,ignore
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct Calculator {
        pub(super) last_result: std::cell::Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Calculator {
        const NAME: &'static str = "MyCalculator";
        type Type = super::Calculator;
    }

    impl ObjectImpl for Calculator {}
}

glib::wrapper! {
    pub struct Calculator(ObjectSubclass<imp::Calculator>);
}

// my_calculator_get_type is automatically generated
#[ffi_impl(prefix = "my")]
impl Calculator {
    // Generates my_calculator_new
    fn new() -> Calculator {
        glib::Object::new::<Calculator>()
    }

    // Generates my_calculator_add
    fn add(&self, a: i32, b: i32) -> i32 {
        let result = a + b;
        self.imp().last_result.set(result);
        result
    }

    // Generates my_calculator_compute, my_calculator_compute_finish and my_calculator_compute_sync
    async fn compute(&self, x: i32, y: i32) -> Result<i32, glib::Error> {
        // Your async computation
        Ok(x * y)
    }

    // Generates my_calculator_get_description
    fn get_description(&self) -> String {
        format!("Last result: {}", self.imp().last_result.load(Ordering::Relaxed))
    }
}
```

## License

MIT
