// Test FFI generation for glib::Enum types
use gobject_macros::{ffi_impl, c_return_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "MyStatus")]
pub enum Status {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

#[ffi_impl(prefix = "my", ty = "enum")]
impl Status {
    #[c_return_type(i32, transfer=primitive)]
    fn idle() -> Status {
        Status::Idle
    }

    #[c_return_type(i32, transfer=primitive)]
    fn running() -> Status {
        Status::Running
    }

    fn is_active(&self) -> bool {
        matches!(self, Status::Running | Status::Paused)
    }

    fn is_done(&self) -> bool {
        matches!(self, Status::Completed | Status::Failed)
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }

    fn can_transition_to(&self, #[c_type(i32, transfer=primitive)] target: Status) -> bool {
        match (*self, target) {
            (Status::Idle, Status::Running) => true,
            (Status::Running, Status::Paused) => true,
            (Status::Paused, Status::Running) => true,
            (Status::Running, Status::Completed) => true,
            (Status::Running, Status::Failed) => true,
            _ => false,
        }
    }

    #[c_return_type(i32, transfer=primitive)]
    fn next(&self) -> Status {
        match self {
            Status::Idle => Status::Running,
            Status::Running => Status::Completed,
            Status::Paused => Status::Running,
            Status::Completed => Status::Idle,
            Status::Failed => Status::Idle,
        }
    }

    // Test async method
    async fn validate_transition(&self, #[c_type(i32, transfer=primitive)] target: Status) -> Result<bool, glib::Error> {
        if self.can_transition_to(target) {
            Ok(true)
        } else {
            Err(glib::Error::new(glib::FileError::Failed, "Invalid transition"))
        }
    }

    // Test fallible sync method
    #[c_return_type(i32, transfer=primitive)]
    fn transition_to(&self, #[c_type(i32, transfer=primitive)] target: Status) -> Result<Status, glib::Error> {
        if self.can_transition_to(target) {
            Ok(target)
        } else {
            Err(glib::Error::new(glib::FileError::Failed, "Invalid transition"))
        }
    }
}

fn main() {
    #[allow(unused_imports)]
    use ffi::MyStatus;
}
