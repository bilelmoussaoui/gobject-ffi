// Test FFI generation for glib::Boxed types
use gobject_macros::{ffi_impl, c_return_type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "MyRectangle")]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
}

#[ffi_impl(prefix = "my", ty="boxed")]
impl Rectangle {
    fn create(x: i32, y: i32, width: i32, height: i32) -> Rectangle {
        Rectangle::new(x, y, width, height)
    }

    fn area(&self) -> i32 {
        self.width * self.height
    }

    fn is_square(&self) -> bool {
        self.width == self.height
    }

    fn to_string(&self) -> String {
        format!("Rectangle({}, {}, {}x{})", self.x, self.y, self.width, self.height)
    }

    // Test async method
    async fn validate(&self) -> Result<bool, glib::Error> {
        Ok(self.width > 0 && self.height > 0)
    }

    // Test fallible sync method
    #[c_return_type(*mut Rectangle, transfer=full)]
    fn scale(&self, factor: i32) -> Result<Rectangle, glib::Error> {
        if factor <= 0 {
            return Err(glib::Error::new(
                glib::FileError::Failed,
                "Scale factor must be positive"
            ));
        }
        Ok(Rectangle::new(self.x, self.y, self.width * factor, self.height * factor))
    }
}

fn main() {
    #[allow(unused_imports)]
    use ffi::MyRectangle;
}
