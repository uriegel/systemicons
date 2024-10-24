use std::{io::Cursor, mem, ptr, slice, thread, time::Duration };
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, FALSE}, Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, GetDIBits, GetObjectW, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, 
            BI_RGB, DIB_RGB_COLORS, HBITMAP, HDC, RGBQUAD
        }, Storage::FileSystem::FILE_ATTRIBUTE_NORMAL, System::{Com::{
                StructuredStorage::CreateStreamOnHGlobal, STATFLAG_DEFAULT, STATSTG
            }, Memory::{self, GlobalLock, GlobalUnlock, GMEM_MOVEABLE}, Ole::{IPicture, OleCreatePictureIndirect, PICTDESC, PICTYPE_ICON}}, UI::{
            Shell::{
                ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON, SHGFI_TYPENAME, SHGFI_USEFILEATTRIBUTES
        }, WindowsAndMessaging::{GetIconInfo, HICON, ICONINFO}}
    }, 
};
use image::{ImageBuffer, ImageFormat, Rgba, RgbaImage};

use crate::Error;

use super::drop::{BitmapDropper, DcDropper, IconDropper};

// TODO extract the correct size from icon

pub fn get_icon(ext: &str, size: i32) -> Result<Vec<u8>, Error> {
    let mut icon = if ext.to_lowercase().ends_with(".exe") {
        let icon = extract_icon(ext, size);
        if !icon.is_invalid() {
            return get_icon_from_exe(icon)
        } 
        if let Some(pos) = ext.find(".exe") {
            get_icon_from_ext(&ext[pos..], size)
        } else {
            icon
        }
    } else {
        get_icon_from_ext(ext, size)
    };
    if icon.is_invalid() {
        icon = extract_icon("C:\\Windows\\system32\\SHELL32.dll", size);
    }
    get_icon_from_hicon(icon)
}

fn get_icon_from_hicon(icon: HICON) -> Result<Vec<u8>, Error> {
    let _icon_dropper = IconDropper(icon);

    let mut pictdesc = PICTDESC {
        cbSizeofstruct: std::mem::size_of::<PICTDESC>() as u32,
        picType: PICTYPE_ICON.0 as u32,
        ..Default::default()
    };

    pictdesc.Anonymous.icon.hicon = icon;

    let res: windows::core::Result<IPicture> = unsafe {
        OleCreatePictureIndirect(&pictdesc, true)
    };

    let hglobal = unsafe { Memory::GlobalAlloc(GMEM_MOVEABLE, 0)? };

    let strom = unsafe { CreateStreamOnHGlobal(hglobal, BOOL(1))? }; // BOOL(1) -> TRUE means stream takes ownership 

    let mut statstg = STATSTG {
        ..Default::default()
    };

    unsafe { res?.SaveAsFile(&strom, BOOL(1))?; }
    unsafe { strom.Stat(&mut statstg, STATFLAG_DEFAULT)? };
    let locked_memory = unsafe { GlobalLock(hglobal) } as *const u8;
    let bytes = unsafe { std::slice::from_raw_parts(locked_memory, statstg.cbSize as usize) };
    let im = image::load_from_memory(&bytes)?; // Assuming bytes contains valid icon data


    let image = im.into_rgba8();
    let modified_image = change_black_to_white(image);

    // Don't call GlobalFree because of deleteOnRelease from CreateStreamOnHGlobal!!!
    let _ = unsafe { GlobalUnlock(hglobal) }; 

    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    modified_image.write_to(&mut cursor, ImageFormat::Png)?;

    Ok(png_bytes)
}

fn change_black_to_white(image: RgbaImage) -> RgbaImage {
    let mut modified_image: RgbaImage = image.clone(); // Create a mutable clone of the image

    for (_x, _y, pixel) in modified_image.enumerate_pixels_mut() {
        let channels = pixel.0;

        // Check if the pixel is black (you can adjust the threshold as needed)
        if channels[0] < 50 && channels[1] < 50 && channels[2] < 50 { // Threshold for black
            *pixel = Rgba([0, 0, 0, 0]); // Set to white, keep original alpha
        }
    }

    modified_image
}


fn get_icon_from_exe(icon: HICON) -> Result<Vec<u8>, Error> {
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
