#[cfg(target_os = "windows")]
use image::ImageError;
#[cfg(target_os = "windows")]
use ::windows::core::Error as WinError;
use std::{fmt, str::Utf8Error};

/// Inner Error type of possible Error
pub enum InnerError {
    IoError(std::io::Error),
    Utf8Error(Utf8Error),
    #[cfg(target_os = "linux")]
    GtkInitError,
    WinResult,
    #[cfg(target_os = "windows")]
    ImageError(ImageError),
    #[cfg(target_os = "windows")]
    WinError(WinError),
}

/// Possible Error
pub struct Error {
    pub message: String,
    pub inner_error: InnerError,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::IoError(error),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::Utf8Error(error),
        }
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::WinResult,
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
            #[cfg(target_os = "linux")]
            &InnerError::GtkInitError => "GtkInitError".to_string(),
            &InnerError::Utf8Error(_) => "Utf8Error".to_string(),
            &InnerError::IoError(_) => "IoError".to_string(),
            #[cfg(target_os = "windows")]
            &InnerError::ImageError(_) => "ImageError".to_string(),
            #[cfg(target_os = "windows")]
            &InnerError::WinError(_) => "WinError".to_string(),
            #[cfg(target_os = "windows")]
            &InnerError::WinResult => "Windows Result".to_string(),
        };
        write!(f, "(Error type: {}", res)
    }
}

