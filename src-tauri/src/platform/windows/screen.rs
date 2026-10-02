//! Copying part of the desktop into memory.
//!
//! A screenshot is taken the way a painting program does it: a memory device
//! context holding a bitmap is told to copy the screen into itself, and the
//! bitmap is then read back as top down pixels. Doing it here rather than
//! through the window manager means nothing is ever put on screen to be seen
//! copying itself, and a region that is covered by another window still shows
//! what the user sees, which is what they aimed at.

use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};

use crate::platform::{screen::Shot, ScreenRect};

/// Most pixels one screenshot may hold, so a nonsense rectangle cannot ask for
/// an allocation the machine cannot make. A region that large is scaled down
/// before it is sent, which is why this is far above the size the service takes.
const MAX_PIXELS: usize = 40_000_000;

/// Copies `rect` of the desktop, in physical pixels.
pub fn grab(rect: ScreenRect) -> Result<Shot, String> {
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return Err("Nothing was selected.".to_string());
    }
    let (width, height) = (width as u32, height as u32);
    if width as usize * height as usize > MAX_PIXELS {
        return Err("That area of the screen is too large to read text from.".to_string());
    }

    unsafe {
        // A null window handle is the desktop, which covers every monitor, so
        // the coordinates of the region are the ones it was asked for.
        let screen = GetDC(None);
        if screen.is_invalid() {
            return Err("The screen could not be read.".to_string());
        }
        let memory = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, width as i32, height as i32);
        if memory.is_invalid() || bitmap.is_invalid() {
            let _ = ReleaseDC(None, screen);
            return Err("The screen could not be read.".to_string());
        }
        let previous = SelectObject(memory, HGDIOBJ(bitmap.0));

        let copied = BitBlt(
            memory,
            0,
            0,
            width as i32,
            height as i32,
            Some(screen),
            rect.left,
            rect.top,
            // CAPTUREBLT includes the layered windows some applications draw
            // themselves with, which are exactly the ones with a lot of text.
            SRCCOPY | CAPTUREBLT,
        );

        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                // Negative, so the rows come back with the top of the picture
                // first, the way every other picture in the crate is written.
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: width * height * 4,
                ..Default::default()
            },
            ..Default::default()
        };
        let read = if copied.is_ok() {
            GetDIBits(
                memory,
                bitmap,
                0,
                height,
                Some(pixels.as_mut_ptr().cast::<c_void>()),
                &mut info,
                DIB_RGB_COLORS,
            )
        } else {
            0
        };

        SelectObject(memory, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(memory);
        let _ = ReleaseDC(None, screen);

        if read == 0 {
            return Err("The screen could not be read.".to_string());
        }
        Ok(Shot {
            origin: (rect.left, rect.top),
            width,
            height,
            pixels,
        })
    }
}
