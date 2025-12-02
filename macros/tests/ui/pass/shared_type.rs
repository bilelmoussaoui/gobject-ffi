// Test FFI generation for glib::Shared types
use gobject_macros::{ffi_impl, c_return_type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointData {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, glib::SharedBoxed)]
#[shared_boxed_type(name = "MyPoint")]
pub struct Point(Arc<PointData>);

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self(Arc::new(PointData { x, y }))
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }
}

#[ffi_impl(c_type_name = "PointData", prefix = "my_point", ty="shared")]
impl Point {
    fn create(x: i32, y: i32) -> Point {
        Point::new(x, y)
    }

    fn distance_from_origin(&self) -> i32 {
        self.0.x.abs() + self.0.y.abs()
    }

    fn is_origin(&self) -> bool {
        self.0.x == 0 && self.0.y == 0
    }

    fn distance_to(&self, other_x: i32, other_y: i32) -> i32 {
        (self.0.x - other_x).abs() + (self.0.y - other_y).abs()
    }

    fn to_string(&self) -> String {
        format!("Point({}, {})", self.0.x, self.0.y)
    }

    fn get_x(&self) -> i32 {
        self.x()
    }

    fn get_y(&self) -> i32 {
        self.y()
    }

    // Test async method
    async fn validate_coordinates(&self) -> Result<bool, glib::Error> {
        Ok(self.0.x >= 0 && self.0.y >= 0)
    }

    // Test fallible sync method
    #[c_return_type(*mut PointData, transfer=none)]
    fn translate(&self, dx: i32, dy: i32) -> Result<Point, glib::Error> {
        let new_x = self.0.x.checked_add(dx)
            .ok_or_else(|| glib::Error::new(glib::FileError::Failed, "X overflow"))?;
        let new_y = self.0.y.checked_add(dy)
            .ok_or_else(|| glib::Error::new(glib::FileError::Failed, "Y overflow"))?;
        Ok(Point::new(new_x, new_y))
    }
}

fn main() {}
