use std::{io::Cursor, mem, ptr, slice, thread, time::Duration };
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::FALSE, Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, GetDIBits, GetObjectW, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, RGBQUAD
        }, Storage::FileSystem::FILE_ATTRIBUTE_NORMAL, UI::{
            Shell::{
                ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON, SHGFI_TYPENAME, SHGFI_USEFILEATTRIBUTES
        }, WindowsAndMessaging::{GetIconInfo, HICON, ICONINFO}}
    },
};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::error::Error;

use super::drop::{BitmapDropper, DcDropper, IconDropper};

// TODO extract the correct size from icon

pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    let mut icon = if ext.to_lowercase().ends_with(".exe") {
        let mut icon = extract_icon(ext, size);
        if icon.is_invalid() {
            if let Some(pos) = ext.find(".exe") {
                icon = get_icon_from_ext(&ext[pos..], size);
            } else {
                icon = extract_icon("C:\\Windows\\system32\\SHELL32.dll", size);
            }
        }
        icon
    } else {
        get_icon_from_ext(ext, size)
    };
    if icon.is_invalid() {
        icon = extract_icon("C:\\Windows\\system32\\SHELL32.dll", size);
    }
    // automatic cleanup:
    let _icon_dropper = IconDropper(icon);

    let mut icon_info = ICONINFO {
        fIcon: FALSE,
        hbmColor: HBITMAP::default(),
        hbmMask: HBITMAP::default(),
        xHotspot: 0,
        yHotspot: 0,
    };
    unsafe { GetIconInfo(icon, &mut icon_info)?; }
    let _info_color_dropper = BitmapDropper(icon_info.hbmColor);
    let _info_mask_dropper = BitmapDropper(icon_info.hbmMask);

    let mut bmp_color = BITMAP::default();
    unsafe {GetObjectW(icon_info.hbmColor, mem::size_of_val(&bmp_color) as i32, Some(&mut bmp_color as *mut _ as *mut _)) }; 

    // Bitmap header setup for the color bitmap
    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: bmp_color.bmWidth,
            biHeight: bmp_color.bmHeight,
            biPlanes: 1,
            biBitCount: 32,    // 32-bit for RGBA
            biCompression: BI_RGB.0,  // No compression
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }; 1], // Initialize an array of RGBQUAD
    };

    let hdc = unsafe {CreateCompatibleDC(HDC::default()) };
    if hdc.is_invalid() {
        return Err("Failed to create compatible DC.".into());
    }
    let _dc_dropper = DcDropper(hdc);

    // Create a DIB section to receive pixel data
    let mut bits_ptr: *mut u8 = ptr::null_mut();
    let hbitmap = unsafe { CreateDIBSection(
        hdc,
        &bitmap_info,
        DIB_RGB_COLORS,
        &mut bits_ptr as *mut *mut u8 as *mut *mut _,
        None,
        0,
    )? };

    if hbitmap.is_invalid() {
        return Err("Failed to create DIB section.".into());
    }
    let _bitmap_dropper = BitmapDropper(hbitmap);

    // Select the DIB section into the device context
    unsafe { SelectObject(hdc, hbitmap) };


    // Get the color bitmap bits
    let success_color = unsafe { GetDIBits(
        hdc,
        icon_info.hbmColor,
        0,
        bmp_color.bmHeight as u32,
        Some(bits_ptr as *mut _),
        &mut bitmap_info,
        DIB_RGB_COLORS,
    ) };

    if success_color == 0 {
        return Err("Failed to get color DIB bits.".into());
    }

    // Copy color bitmap bits to a separate buffer
    let pixels_color = unsafe { slice::from_raw_parts(bits_ptr, (bmp_color.bmWidth * bmp_color.bmHeight * 4) as usize).to_vec() };

    // Get the mask bitmap bits (monochrome)
    let success_mask = unsafe { GetDIBits(
        hdc,
        icon_info.hbmMask,
        0,
        bmp_color.bmHeight as u32,
        Some(bits_ptr as *mut _),
        &mut bitmap_info,
        DIB_RGB_COLORS,
    ) };

    if success_mask == 0 {
        return Err("Failed to get mask DIB bits.".into());
    }

    // Copy mask bitmap bits to a separate buffer
    let pixels_mask = unsafe { slice::from_raw_parts(bits_ptr, (bmp_color.bmWidth * bmp_color.bmHeight * 4) as usize).to_vec() };

    // Combine color and mask bitmaps to handle transparency
    let mut final_pixels: Vec<u8> = Vec::with_capacity((bmp_color.bmWidth * bmp_color.bmHeight * 4) as usize);
    for row in (0..bmp_color.bmHeight).rev() { // Reverse row order to handle bottom-up storage
        for col in 0..bmp_color.bmWidth  {
            let i = (row * bmp_color.bmWidth + col) as usize;
            let color_idx = i * 4;
            let mask_idx = i * 4;

            // The mask is typically monochrome (1-bit per pixel), but for convenience, we assume it's 32-bit.
            let is_transparent = pixels_mask[mask_idx] == 0xFF;  // Fully transparent if mask is white

            if is_transparent {
                final_pixels.extend_from_slice(&[0, 0, 0, 0]);  // Transparent pixel (RGBA)
            } else {
                // Correct color channels from BGR to RGB
                final_pixels.push(pixels_color[color_idx + 2]); // Red
                final_pixels.push(pixels_color[color_idx + 1]); // Green
                final_pixels.push(pixels_color[color_idx + 0]); // Blue
                final_pixels.push(0xFF);                        // Opaque alpha
            }
        }
    }
    // Create the image buffer from the final combined data
    let img_buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(bmp_color.bmWidth as u32, bmp_color.bmHeight as u32, final_pixels).ok_or("Failed to create image buffer.")?;

    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    img_buffer.write_to(&mut cursor, ImageFormat::Png)?;

    Ok(png_bytes)
}

fn get_icon_from_ext(ext: &str, size: i32) -> HICON {
    let p_path = utf_16_null_terminated(ext);
    let mut file_info = SHFILEINFOW {
        dwAttributes: 0,
        hIcon: HICON::default(),
        iIcon: 0,
        szDisplayName: [0; 260],
        szTypeName: [0; 80],
    };
    let file_info_size = mem::size_of_val(&file_info) as u32;
    for _ in 0..3 {
        unsafe { SHGetFileInfoW(
            PCWSTR(p_path.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut file_info),
            file_info_size,
            SHGFI_ICON | SHGFI_USEFILEATTRIBUTES | SHGFI_TYPENAME
                | if size > 16 {
                    SHGFI_LARGEICON
                } else {
                    SHGFI_SMALLICON
                },
        ) };
        if !file_info.hIcon.is_invalid() {
            break;
        } else {
            let millis = Duration::from_millis(30);
            thread::sleep(millis);
        }
    }
    file_info.hIcon
}


fn extract_icon(path: &str, size: i32) -> HICON {
    let mut icons: Vec<HICON> = vec![HICON::default(); 1];  

    let path = utf_16_null_terminated(path);
    unsafe { ExtractIconExW(
        PCWSTR(path.as_ptr()),
        0,
        if size > 16 { Some(icons.as_mut_ptr()) } else { None },
        if size <= 16 { Some(icons.as_mut_ptr()) } else { None },
        1,
    )};
    icons[0]
}

fn utf_16_null_terminated(x: &str) -> Vec<u16> {
    x.encode_utf16().chain(std::iter::once(0)).collect()
}

