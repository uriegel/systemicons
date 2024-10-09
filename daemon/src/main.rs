use std::{fmt, path::PathBuf};

use gtk4::prelude::*;
use gtk4::{gdk::Display, gio::{self, ThemedIcon}, IconLookupFlags, IconTheme, TextDirection};

fn main() {
    gtk4::init().unwrap();
    let file = get_icon_as_file(".mp4", 16).unwrap();
    println!("{}", file);
}

fn get_icon_as_file(ext: &str, size: i32) -> Result<String, Error> {
    let display = Display::default()
        .ok_or(Error{ inner_error: InnerError::GtkInitError, message: "Could not get default display".to_string()})?;
    let theme = IconTheme::for_display(&display); 
    let mime = gio::content_type_guess(Some(&PathBuf::from(ext)) , &[]);   
    
    
    
    println!("MIME {}", mime.0.to_string());
    let icon = gio::content_type_get_icon(&mime.0);
    let icon_names = get_icon_names(&icon).unwrap();
    let icon_names: Vec<&str> = icon_names.iter().map(|i| i.as_str()).collect();
    let themed_icon = ThemedIcon::from_names(icon_names.as_slice());
    let affe = theme.lookup_by_gicon(&themed_icon, size, 1, TextDirection::None, IconLookupFlags::FORCE_REGULAR);
    let feile = affe.file().unwrap();
    let pfad = feile.path().unwrap();
    Ok(pfad.to_string_lossy().to_string())
}

fn get_icon_names(icon: &gio::Icon) -> Option<Vec<String>> {
    // Try to downcast the GIcon to a ThemedIcon
    if let Some(themed_icon) = icon.dynamic_cast_ref::<ThemedIcon>() {
        // Retrieve the icon names
        let icon_names = themed_icon.names();
        Some(icon_names.iter().map(|i| i.to_string()).collect())
    } else {
        None
    }
}

pub enum InnerError {
    IoError(std::io::Error),
    GtkInitError,
    #[cfg(target_os = "windows")]
    ImageError(ImageError),
    Generic
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

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.message, self.inner_error)
    }
}

impl fmt::Debug for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            &InnerError::GtkInitError => "GtkInit".to_string(),
            &InnerError::IoError(_) => "Io".to_string(),
            #[cfg(target_os = "windows")]
            &InnerError::ImageError(_) => "Image".to_string(),
            &InnerError::Generic => "Generic".to_string()
        };
        write!(f, "(Error type: {}", res)
    }
}


