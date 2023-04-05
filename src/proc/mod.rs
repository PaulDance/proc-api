use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProcInfo {
    pub pid: i32,
    pub uid: u32,
    pub name: String,
    pub user: String,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;
