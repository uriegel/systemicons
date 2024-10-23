pub mod request;
mod drop;

use image::ImageError;
use windows::core::Error as WinError;

use crate::error::{Error, InnerError};

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::ImageError(error),
        }
    }
}

impl From<WinError> for Error {
    fn from(error: WinError) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::WinError(error),
        }
    }
}
