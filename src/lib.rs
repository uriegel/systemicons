//! # systemicons
//! 
//! With this lib you can retrive the system icon which is associated 
//! to a certain file extension. The icon will be in the .png format. 
//! Windows and Linux (GTK) are supperted.
use std::{fmt, str::Utf8Error};
#[cfg(target_os = "windows")]
use image::ImageError;

/// Inner Error type of possible Error
pub enum InnerError {
    IoError(std::io::Error),
    Utf8Error(Utf8Error),
    GtkInitError,
    #[cfg(target_os = "windows")]
    ImageError(ImageError)
}

/// Possible Error
pub struct Error {
    pub message: String,
    pub inner_error: InnerError 
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::IoError(error)
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::Utf8Error(error)
        }
    }
}

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::ImageError(error)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.message, self.inner_error)
    }
}

impl fmt::Debug for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            &InnerError::GtkInitError => "GtkInitError".to_string(),
            &InnerError::Utf8Error(_) => "Utf8Error".to_string(),
            &InnerError::IoError(_) => "IoError".to_string(),
            &InnerError::ImageError(_) => "ImageError".to_string()
        };
        write!(f, "(Error type: {}", res)
    }
}

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Retrieving system icon. You have to specify the file extension and deisred icon size (like 16, 32 or 64).
/// Returns the icon formatted as png as byte buffer.
#[cfg(target_os = "linux")]
pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    linux::request::get_icon(ext, size)
}
#[cfg(target_os = "windows")]
pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    windows::request::get_icon(ext, size)
}