use image::ImageError;

use crate::{Error, InnerError};

pub mod request;

// impl From<windows_result::error::Error> for Error {
//     fn from(error: windows_result::error::Error) -> Self {
//         Error {
//             message: error.to_string(),
//             inner_error: windows_result::error::Error(error),
//         }
//     }
// }

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Self {
        Error {
            message: error.to_string(),
            inner_error: InnerError::ImageError(error),
        }
    }
}


