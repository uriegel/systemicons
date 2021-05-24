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