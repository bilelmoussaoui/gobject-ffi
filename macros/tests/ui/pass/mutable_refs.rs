// Test mutable reference parameters
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct MutHandler;

    #[glib::object_subclass]
    impl ObjectSubclass for MutHandler {
        const NAME: &'static str = "MutHandler";
        type Type = super::MutHandler;
    }

    impl ObjectImpl for MutHandler {}
}

glib::wrapper! {
    pub struct MutHandler(ObjectSubclass<imp::MutHandler>);
}

#[ffi_impl(prefix = "mut_handler")]
impl MutHandler {
    // Sync mutable references
    fn increment(&self, value: &mut i32) {
        *value += 1;
    }

    fn double(&self, value: &mut u64) {
        *value *= 2;
    }

    fn modify_multiple(&self, a: &mut i32, b: &mut u64) {
        *a += 5;
        *b *= 3;
    }

    // Async mutable references
    async fn async_increment(&self, value: &mut i32) {
        *value += 10;
    }

    async fn async_modify(&self, value: &mut i64) {
        *value = -*value;
    }
}

fn main() {}
