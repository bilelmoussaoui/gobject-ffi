// Test constructor method generation
use glib::subclass::prelude::*;
use gobject_macros::ffi_impl;

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct Widget;

    #[glib::object_subclass]
    impl ObjectSubclass for Widget {
        const NAME: &'static str = "Widget";
        type Type = super::Widget;
    }

    impl ObjectImpl for Widget {}
}

glib::wrapper! {
    pub struct Widget(ObjectSubclass<imp::Widget>);
}

#[ffi_impl]
impl Widget {
    // Sync constructors
    fn new() -> Widget {
        glib::Object::new::<Widget>()
    }

    fn with_name(name: String) -> Widget {
        let _ = name;
        glib::Object::new::<Widget>()
    }

    fn try_new() -> Result<Widget, glib::Error> {
        Ok(glib::Object::new::<Widget>())
    }

    // Async constructors
    async fn new_async() -> Widget {
        glib::Object::new::<Widget>()
    }

    async fn try_new_async(name: String) -> Result<Widget, glib::Error> {
        let _ = name;
        Ok(glib::Object::new::<Widget>())
    }
}

fn main() {}
