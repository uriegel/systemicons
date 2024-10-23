use std::{io::Cursor, mem, ptr, thread, time::Duration };
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::FALSE, Graphics::Gdi::{
            DeleteObject, GetBitmapBits, GetObjectW, BITMAP, BITMAPINFOHEADER, HBITMAP
        }, Storage::FileSystem::FILE_ATTRIBUTE_NORMAL, UI::{
            Shell::{
                ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON, SHGFI_TYPENAME, SHGFI_USEFILEATTRIBUTES
        }, WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO}}
    },
};
use image::ImageFormat;

use crate::Error;

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

    let mut icon_info = ICONINFO {
        fIcon: FALSE,
        hbmColor: HBITMAP::default(),
        hbmMask: HBITMAP::default(),
        xHotspot: 0,
        yHotspot: 0,
    };
    unsafe {
        GetIconInfo(icon, &mut icon_info)?;
        let _ = DestroyIcon(icon);
    }

    let mut bmp_color = BITMAP {
        bmBits: ptr::null_mut(),
        bmBitsPixel: 0,
        bmHeight: 0,
        bmPlanes: 0,
        bmType: 0,
        bmWidth: 0,
        bmWidthBytes: 0,
    };
    unsafe {GetObjectW(icon_info.hbmColor, mem::size_of_val(&bmp_color) as i32, Some(&mut bmp_color as *mut _ as *mut _)) }; 

    let mut bmp_mask = BITMAP {
        bmBits: ptr::null_mut(),
        bmBitsPixel: 0,
        bmHeight: 0,
        bmPlanes: 0,
        bmType: 0,
        bmWidth: 0,
        bmWidthBytes: 0,
    };
    unsafe {GetObjectW(icon_info.hbmMask, mem::size_of_val(&bmp_mask) as i32, Some(&mut bmp_mask as *mut _ as *mut _)) };

    fn get_bitmap_count(bitmap: &BITMAP)->i32 {
        let mut n_width_bytes = bitmap.bmWidthBytes;
        // bitmap scanlines MUST be a multiple of 4 bytes when stored
        // inside a bitmap resource, so round up if necessary
        if n_width_bytes & 3 != 0 {
            n_width_bytes = (n_width_bytes + 4) & !3;
        }
    
        n_width_bytes * bitmap.bmHeight
    }

    let icon_header_size = mem::size_of::<ICONHEADER>();
    let icon_dir_size = mem::size_of::<ICONDIR>();
    let info_header_size = mem::size_of::<BITMAPINFOHEADER>();
    let bitmap_bytes_count = get_bitmap_count(&bmp_color) as usize;
    let mask_bytes_count = get_bitmap_count(&bmp_mask) as usize;

    let complete_size = icon_header_size + icon_dir_size + info_header_size + bitmap_bytes_count + mask_bytes_count;

    let image_bytes_count = bitmap_bytes_count + mask_bytes_count;
    let mut bytes = Vec::<u8>::with_capacity(complete_size);
    unsafe { bytes.set_len(complete_size) };

    let iconheader = ICONHEADER { 
        id_reserved: 0, 
        id_type: 1, // Type 1 = ICON (type 2 = CURSOR)
        id_count: 1, // number of ICONDIRs
    };
    let byte_ptr: *mut u8 = unsafe {mem::transmute(&iconheader) };
    unsafe {ptr::copy_nonoverlapping(byte_ptr, bytes.as_mut_ptr(), icon_header_size)}; 
    let pos = icon_header_size;

    let color_count = if bmp_color.bmBitsPixel >= 8 { 
        0 
    } else { 
        1 << (bmp_color.bmBitsPixel * bmp_color.bmPlanes) 
    };

    // Create the ICONDIR structure
    let icon_dir = ICONDIR {
        b_width: bmp_color.bmWidth as u8,
        b_height: bmp_color.bmHeight as u8,
        b_color_count: color_count,
        b_reserved: 0,
        w_planes: bmp_color.bmPlanes,
        w_bit_count: bmp_color.bmBitsPixel,
        dw_image_offset: (icon_header_size + 16) as u32,
        dw_bytes_in_res: (mem::size_of::<BITMAPINFOHEADER>() + image_bytes_count) as u32
    };

    let byte_ptr: *mut u8 = unsafe { mem::transmute(&icon_dir) };
    unsafe { ptr::copy_nonoverlapping(byte_ptr, bytes[pos..].as_mut_ptr(), icon_dir_size) }; 
    let pos = pos + icon_dir_size;

    let bi_header = BITMAPINFOHEADER {
        biSize: info_header_size as u32,
        biWidth: bmp_color.bmWidth,
        biHeight: bmp_color.bmHeight * 2, // height of color+mono
        biPlanes: bmp_color.bmPlanes,
        biBitCount: bmp_color.bmBitsPixel,
        biSizeImage: image_bytes_count as u32,
        biClrImportant: 0,
        biClrUsed: 0,
        biCompression: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0
    };
    let byte_ptr: *mut u8 = unsafe {mem::transmute(&bi_header) };
    unsafe { ptr::copy_nonoverlapping(byte_ptr, bytes[pos..].as_mut_ptr(), info_header_size) }; 
    let pos = pos + info_header_size;

    // write the RGBQUAD color table (for 16 and 256 colour icons)
    if bmp_color.bmBitsPixel == 2 || bmp_color.bmBitsPixel == 8 {}        

    write_icon_data_to_memory(&mut bytes[pos..], icon_info.hbmColor, 
        &bmp_color, bitmap_bytes_count as usize);
    let pos = pos + bitmap_bytes_count as usize;
    write_icon_data_to_memory(&mut bytes[pos..], icon_info.hbmMask, 
        &bmp_mask, mask_bytes_count as usize);

    let im = image::load_from_memory(&bytes)?; // Assuming bytes contains valid icon data
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    im.write_to(&mut cursor, ImageFormat::Png)?;

    unsafe {
        let _ = DeleteObject(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmMask);
    }

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

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ICONHEADER {
    id_reserved: i16, 
    id_type: i16,
    id_count: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ICONDIR {
    b_width: u8,
    b_height: u8,
    b_color_count: u8,
    b_reserved: u8,
    w_planes: u16, // for cursors, this field = wXHotSpot
    w_bit_count: u16, // for cursors, this field = wYHotSpot
    dw_bytes_in_res: u32,
    dw_image_offset: u32, // file-offset to the start of ICONIMAGE
}

fn write_icon_data_to_memory(mem: &mut [u8], h_bitmap: HBITMAP, bmp: &BITMAP, bitmap_byte_count: usize) {
    unsafe {
        let mut icon_data = Vec::<u8>::with_capacity(bitmap_byte_count);
        icon_data.set_len(bitmap_byte_count);

        GetBitmapBits(h_bitmap, bitmap_byte_count as i32, icon_data.as_mut_ptr() as *mut _);

        // bitmaps are stored inverted (vertically) when on disk..
        // so write out each line in turn, starting at the bottom + working
        // towards the top of the bitmap. Also, the bitmaps are stored in packed
        // in memory - scanlines are NOT 32bit aligned, just 1-after-the-other
        let mut pos = 0;
        for i in (0..bmp.bmHeight).rev() {
            // Write the bitmap scanline
            
            ptr::copy_nonoverlapping(icon_data[(i * bmp.bmWidthBytes) as usize..].as_ptr(), mem[pos..].as_mut_ptr(), bmp.bmWidthBytes as usize); // 1 line of BYTES
            pos += bmp.bmWidthBytes as usize;

            // extend to a 32bit boundary (in the file) if necessary
            if bmp.bmWidthBytes & 3 != 0 {
                let padding: [u8; 4] = [0; 4];
                ptr::copy_nonoverlapping(padding.as_ptr(), mem[pos..].as_mut_ptr(), (4 - bmp.bmWidthBytes) as usize); 
                pos += 4 - bmp.bmWidthBytes as usize;
            }
        }
    }
}

fn utf_16_null_terminated(x: &str) -> Vec<u16> {
    x.encode_utf16().chain(std::iter::once(0)).collect()
}
