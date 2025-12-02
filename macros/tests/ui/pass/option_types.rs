// Test Option<T> parameter and return type handling
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct OptionHandler;

    #[glib::object_subclass]
    impl ObjectSubclass for OptionHandler {
        const NAME: &'static str = "OptionHandler";
        type Type = super::OptionHandler;
    }

    impl ObjectImpl for OptionHandler {}
}

glib::wrapper! {
    pub struct OptionHandler(ObjectSubclass<imp::OptionHandler>);
}

#[ffi_impl(prefix = "option_handler")]
impl OptionHandler {
    // Option parameters
    fn process_optional_string(&self, text: Option<String>) -> bool {
        text.is_some()
    }

    fn get_length(&self, text: Option<String>) -> i32 {
        text.map_or(0, |s| s.len() as i32)
    }

    // Option return types
    fn maybe_string(&self, flag: bool) -> Option<String> {
        if flag {
            Some("result".to_string())
        } else {
            None
        }
    }

    fn find_value(&self, _key: String) -> Option<i32> {
        Some(42)
    }

    async fn async_maybe_string(&self) -> Option<String> {
        Some("async result".to_string())
    }

    async fn try_maybe_string(&self) -> Result<Option<String>, glib::Error> {
        Ok(Some("result".to_string()))
    }
}

fn main() {}
