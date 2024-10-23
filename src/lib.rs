//! # systemicons
//!
//! With this lib you can retrieve the system icon which is associated
//! to a certain file extension. The icon will be in the .png format.
//! Windows and Linux (GTK) are supported.
//!
//! When you specify an absolute path to a .exe file, then the icon is loaded from resource, if the exe contains an icon resource.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub mod error;

use error::Error;

/// Retrieving system icon. You have to specify the file extension and desired icon size (like 16, 32 or 64).
/// Returns the icon formatted as png as byte buffer.
#[cfg(target_os = "linux")]
pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    crate::linux::request::get_icon(ext, size)
}
#[cfg(target_os = "windows")]
pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    crate::windows::request::get_icon(ext, size)
}

/// Retrieving system icon. You have to specify the file extension and desired icon size (like 16, 32 or 64).
/// Returns the path to the system icon.
#[cfg(target_os = "linux")]
pub fn get_icon_as_file(ext: &str, size: i32) -> Result<String, Error> {
    crate::linux::request::get_icon_as_file(ext, size)
}

/// In a non GTK program you have to initialize GTK when getting system icons (Linux)-
#[cfg(target_os = "linux")]
pub fn init() {
    crate::linux::request::init()
}

/// Retrieving system icon. You have to specify the file extension and desired icon size (like 16, 32 or 64).
/// Returns the icon formatted as png as byte buffer.
#[cfg(target_os = "macos")]
pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    crate::macos::request::get_icon(ext, size.into())
}

/// Retrieving system icon. You have to specify the file extension and desired icon size (like 16, 32 or 64).
/// Returns the path to the system icon.
#[cfg(target_os = "macos")]
pub fn get_icon_as_file(ext: &str, size: i32) -> Result<String, Error> {
    crate::macos::request::get_icon_as_file(ext, size.into())
}
