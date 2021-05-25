//! # systemicons
//! 
//! With this lib you can retrive the system icon which is associated 
//! to a certain file extension. The icon will be in the .png format. 
//! Windows and Linux (GTK) are supperted.
//!
//! # !!!Under Construction!!!

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn get_icon(ext: &str, size: i32) -> String { 
    linux::request::get_icon(ext, size)
}