// Tests for the FFI generation macros
//
// This test verifies that the macro generates valid Rust code for all supported
// scenarios. The test passes if the code compiles successfully.

use glib::subclass::prelude::*;
use gobject_macros::{c_return_type, ffi_impl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "TestStatus")]
enum Status {
    Idle,
    Running,
    Completed,
    Failed,
}

#[glib::flags(name = "TestPermissions")]
enum Permissions {
    #[flags_value(name = "Read")]
    READ = 0b0001,
    #[flags_value(name = "Write")]
    WRITE = 0b0010,
    #[flags_value(name = "Execute")]
    EXECUTE = 0b0100,
}

mod imp {
    use super::*;
    #[derive(Default)]
    pub struct TestObject;

    #[glib::object_subclass]
    impl ObjectSubclass for TestObject {
        const NAME: &'static str = "TestObject";
        type Type = super::TestObject;
    }

    impl ObjectImpl for TestObject {}
}

glib::wrapper! {
    pub struct TestObject(ObjectSubclass<imp::TestObject>);
}

#[ffi_impl]
impl TestObject {
        // Bool variants
        async fn async_fallible_bool(&self) -> Result<bool, glib::Error> {
            Ok(true)
        }
        async fn async_infallible_bool(&self) -> bool {
            false
        }
        fn sync_fallible_bool(&self) -> Result<bool, glib::Error> {
            Ok(true)
        }
        fn sync_infallible_bool(&self) -> bool {
            false
        }

        // String variants
        async fn async_fallible_string(&self) -> Result<String, glib::Error> {
            Ok("test".to_string())
        }
        async fn async_infallible_string(&self) -> String {
            "test".to_string()
        }
        fn sync_fallible_string(&self) -> Result<String, glib::Error> {
            Ok("test".to_string())
        }
        fn sync_infallible_string(&self) -> String {
            "test".to_string()
        }

        // Numeric types
        fn add_i32(&self, a: i32, b: i32) -> i32 {
            a + b
        }
        async fn get_u32(&self) -> u32 {
            42
        }
        fn multiply_i64(&self, a: i64, b: i64) -> Result<i64, glib::Error> {
            Ok(a * b)
        }
        async fn get_u64(&self) -> u64 {
            1234567890
        }
        fn divide_f32(&self, a: f32, b: f32) -> f32 {
            a / b
        }
        async fn calculate_f64(&self, x: f64) -> Result<f64, glib::Error> {
            Ok(x * 2.5)
        }

        // Void return
        fn do_something(&self) {}
        async fn do_something_async(&self) {}

        // Bytes
        async fn async_get_bytes(&self) -> Result<glib::Bytes, glib::Error> {
            Ok(glib::Bytes::from_static(b"test data"))
        }
        async fn async_infallible_bytes(&self) -> glib::Bytes {
            glib::Bytes::from_static(b"infallible data")
        }
        async fn process_bytes(&self, data: Vec<u8>) -> Result<glib::Bytes, glib::Error> {
            Ok(glib::Bytes::from_owned(data))
        }

        // Variant and PathBuf
        fn process_variant(&self, data: glib::Variant) -> Result<glib::Variant, glib::Error> {
            Ok(data)
        }
        async fn get_config(&self) -> glib::Variant {
            glib::Variant::from("test_value")
        }
        fn get_file_path(&self, dir: std::path::PathBuf) -> std::path::PathBuf {
            dir.join("file.txt")
        }
        async fn find_path(&self) -> Result<std::path::PathBuf, glib::Error> {
            Ok(std::path::PathBuf::from("/tmp/test"))
        }

        // Option<T> and Vec<String>
        fn process_optional_string(&self, text: Option<String>) -> Option<String> {
            text.map(|s| s.to_uppercase())
        }
        async fn get_optional_path(&self) -> Result<Option<std::path::PathBuf>, glib::Error> {
            Ok(Some(std::path::PathBuf::from("/tmp/test")))
        }
        fn join_strings(&self, items: Vec<String>) -> String {
            items.join(", ")
        }
        async fn get_string_list(&self) -> Vec<String> {
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        }

        // Constructors
        async fn new_async(name: String) -> Result<TestObject, glib::Error> {
            let _ = name;
            Ok(glib::Object::new::<TestObject>())
        }
        async fn new_async_infallible() -> TestObject {
            glib::Object::new::<TestObject>()
        }
        fn new_sync(name: String) -> Result<TestObject, glib::Error> {
            let _ = name;
            Ok(glib::Object::new::<TestObject>())
        }
        fn new_sync_infallible() -> TestObject {
            glib::Object::new::<TestObject>()
        }

        // Parameter variations
        async fn no_params(&self) -> Result<String, glib::Error> {
            Ok("test".to_string())
        }
        async fn many_params(&self, a: String, b: bool, c: String) -> Result<String, glib::Error> {
            Ok(format!("{}-{}-{}", a, b, c))
        }

        // Enums and Flags
        fn check_status(&self, #[c_type(i32, transfer=primitive)] status: Status) -> bool {
            status == Status::Running
        }
        #[c_return_type(i32, transfer=primitive)]
        async fn get_status(&self) -> Status {
            Status::Completed
        }
        fn process_with_status(
            &self,
            input: String,
            #[c_type(i32, transfer=primitive)] status: Status,
        ) -> Result<String, glib::Error> {
            Ok(format!("{}: {:?}", input, status))
        }
        #[c_return_type(i32, transfer=primitive)]
        async fn async_get_status(
            &self,
            #[c_type(i32, transfer=primitive)] initial: Status,
        ) -> Result<Status, glib::Error> {
            let _ = initial;
            Ok(Status::Failed)
        }

        fn check_permissions(&self, #[c_type(u32, transfer=primitive)] perms: Permissions) -> bool {
            perms.contains(Permissions::READ)
        }
        #[c_return_type(u32, transfer=primitive)]
        async fn get_permissions(&self) -> Permissions {
            Permissions::READ | Permissions::WRITE
        }
        #[c_return_type(u32, transfer=primitive)]
        fn grant_permissions(
            &self,
            #[c_type(u32, transfer=primitive)] current: Permissions,
            #[c_type(u32, transfer=primitive)] new: Permissions,
        ) -> Permissions {
            current | new
        }
        async fn async_check_permissions(
            &self,
            #[c_type(u32, transfer=primitive)] perms: Permissions,
        ) -> Result<bool, glib::Error> {
            Ok(perms.contains(Permissions::EXECUTE))
        }

        // Mutable references (primitives only)
        fn increment(&self, value: &mut i32) {
            *value += 1;
        }
        fn double(&self, value: &mut u64) {
            *value *= 2;
        }
        async fn async_increment(&self, value: &mut i32) {
            *value += 10;
        }
        fn modify_multiple(&self, a: &mut i32, b: &mut u64) {
            *a += 5;
            *b *= 3;
        }

        // External enum from glib/gio (using #[c_type] attribute)
        // Since we can't implement FfiConvert for external types, we use #[c_type]
        fn set_bus_type(&self, #[c_type(i32, transfer=primitive)] bus_type: gio::BusType) -> bool {
            !matches!(bus_type, gio::BusType::None)
        }

        #[c_return_type(i32, transfer=primitive)]
        async fn async_get_bus_type(&self) -> Result<gio::BusType, glib::Error> {
            Ok(gio::BusType::Session)
        }
}

fn main() {}
