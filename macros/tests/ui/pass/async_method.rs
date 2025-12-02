// Test async method generation
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct AsyncService;

    #[glib::object_subclass]
    impl ObjectSubclass for AsyncService {
        const NAME: &'static str = "AsyncService";
        type Type = super::AsyncService;
    }

    impl ObjectImpl for AsyncService {}
}

glib::wrapper! {
    pub struct AsyncService(ObjectSubclass<imp::AsyncService>);
}

#[ffi_impl(prefix = "async_service")]
impl AsyncService {
    // Async infallible
    async fn fetch_data(&self) -> String {
        "data".to_string()
    }

    async fn compute(&self, value: i32) -> i32 {
        value * 2
    }

    async fn void_async(&self) {
        // Does nothing
    }

    // Async fallible
    async fn try_fetch(&self) -> Result<String, glib::Error> {
        Ok("data".to_string())
    }

    async fn try_compute(&self, value: i32) -> Result<i32, glib::Error> {
        Ok(value * 2)
    }

    async fn try_void(&self) -> Result<(), glib::Error> {
        Ok(())
    }
}

fn main() {}
