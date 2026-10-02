//! A picture of part of the desktop, and what is done with one.
//!
//! Taking the picture is a platform call and lives in the implementation, which
//! is why [`grab`] is re-exported from here; cutting a region out of one and
//! encoding it for the network is arithmetic on plain bytes, so it belongs to
//! the portable side of the layer.

use super::ScreenRect;

/// Smallest picture the OCR service still accepts: it refuses anything with a
/// side below this, and a screenshot that small holds no text anyway.
pub const MIN_SIDE: u32 = 15;

/// Longest side the OCR service accepts, in pixels. Anything larger is scaled
/// down before it is sent, however little of it there is to compress.
pub const MAX_SIDE: u32 = 4096;

/// A rectangle of the desktop, as it looked when it was copied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shot {
    /// Physical position of the top left pixel on the desktop.
    pub origin: (i32, i32),
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` bytes of BGRA, top row first.
    pub pixels: Vec<u8>,
}

impl Shot {
    /// The part of the picture inside `rect`, in desktop coordinates.
    pub fn crop(&self, rect: ScreenRect) -> Result<Shot, String> {
        let left = rect.left - self.origin.0;
        let top = rect.top - self.origin.1;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err("Nothing was selected.".to_string());
        }
        let (width, height) = (width as u32, height as u32);
        if left < 0
            || top < 0
            || left as u32 + width > self.width
            || top as u32 + height > self.height
        {
            return Err("The selected area is not on the screen any more.".to_string());
        }

        let mut pixels = Vec::with_capacity((width as usize) * (height as usize) * 4);
        for row in 0..height {
            let start = (((top as u32) + row) as usize * self.width as usize + left as usize) * 4;
            let end = start + (width as usize) * 4;
            pixels.extend_from_slice(&self.pixels[start..end]);
        }
        Ok(Shot {
            origin: (rect.left, rect.top),
            width,
            height,
            pixels,
        })
    }

    /// Encodes the picture as a PNG of at most `byte_limit` bytes.
    ///
    /// A region of text is a few hundred kilobytes, but a whole screen of a
    /// photo is several megabytes and the relay refuses those, so the picture is
    /// halved until it fits. Text stays readable through that, and a request the
    /// service would have thrown away becomes a translation.
    pub fn to_png(&self, byte_limit: usize) -> Result<Vec<u8>, String> {
        let mut shot = self.clone();
        loop {
            let bytes = shot.encode()?;
            let fits = bytes.len() <= byte_limit
                && shot.width.max(shot.height) <= MAX_SIDE
                && shot.width.min(shot.height) >= MIN_SIDE;
            let small = shot.width < MIN_SIDE * 4 || shot.height < MIN_SIDE * 4;
            if fits || small {
                return Ok(bytes);
            }
            shot = shot.halved();
        }
    }

    /// Every pixel of the picture, so the encoder can write it out.
    fn rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.pixels.len());
        // The screen hands out BGRA and its alpha is not the picture's, so the
        // blue and red are swapped and every pixel is written as opaque.
        for pixel in self.pixels.chunks_exact(4) {
            rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], 255]);
        }
        rgba
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("Could not encode the screenshot: {error}"))?;
        writer
            .write_image_data(&self.rgba())
            .map_err(|error| format!("Could not encode the screenshot: {error}"))?;
        drop(writer);
        Ok(out)
    }

    /// The picture at half its size, average of every two by two block.
    fn halved(&self) -> Shot {
        let width = (self.width / 2).max(1);
        let height = (self.height / 2).max(1);
        let mut pixels = Vec::with_capacity((width as usize) * (height as usize) * 4);
        for row in 0..height {
            for column in 0..width {
                let mut sum = [0u32; 3];
                for dy in 0..2 {
                    for dx in 0..2 {
                        let x = (column * 2 + dx).min(self.width - 1);
                        let y = (row * 2 + dy).min(self.height - 1);
                        let at = ((y * self.width + x) as usize) * 4;
                        for (value, byte) in sum.iter_mut().zip(&self.pixels[at..at + 3]) {
                            *value += u32::from(*byte);
                        }
                    }
                }
                for channel in sum {
                    pixels.push((channel / 4) as u8);
                }
                pixels.push(255);
            }
        }
        Shot {
            origin: self.origin,
            width,
            height,
            pixels,
        }
    }
}

/// The rectangle of the monitor that contains `(x, y)`, with its scale factor.
///
/// The screenshot overlay covers one screen and not the whole desktop, because
/// a window spanning several monitors of different scaling would have one scale
/// factor for all of them and report the region it was dragged in the wrong
/// coordinates. Reading the text of a region the user picks on the screen they
/// are looking at never needs more than one monitor.
pub fn monitor_of(monitors: &[(ScreenRect, f64)], x: i32, y: i32) -> Option<(ScreenRect, f64)> {
    monitors
        .iter()
        .find(|(rect, _)| x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom)
        .copied()
}

#[cfg(windows)]
pub use super::windows::screen::grab;

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture with the byte at `(x, y)` standing for its position.
    fn shot(width: u32, height: u32) -> Shot {
        let mut pixels = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let value = (y * width + x) as u8;
                pixels.extend_from_slice(&[value, 0, 0, 0]);
            }
        }
        Shot {
            origin: (100, 200),
            width,
            height,
            pixels,
        }
    }

    #[test]
    fn a_crop_keeps_the_pixels_of_the_region_it_asked_for() {
        let full = shot(4, 3);
        let part = full
            .crop(ScreenRect {
                left: 101,
                top: 201,
                right: 103,
                bottom: 203,
            })
            .unwrap();

        assert_eq!((part.width, part.height), (2, 2));
        assert_eq!(part.origin, (101, 201));
        // Rows one and two, columns one and two of the original.
        assert_eq!(
            part.pixels,
            vec![5, 0, 0, 0, 6, 0, 0, 0, 9, 0, 0, 0, 10, 0, 0, 0]
        );
    }

    #[test]
    fn a_crop_outside_the_picture_is_refused() {
        let full = shot(4, 3);
        // Past the right hand edge, and a region that is empty.
        let past = full.crop(ScreenRect {
            left: 102,
            top: 200,
            right: 105,
            bottom: 202,
        });
        assert!(past.is_err());
        let empty = full.crop(ScreenRect {
            left: 101,
            top: 201,
            right: 101,
            bottom: 201,
        });
        assert!(empty.is_err());
    }

    #[test]
    fn the_monitor_the_point_falls_on_is_the_one_that_answers() {
        let screens = vec![
            (
                ScreenRect {
                    left: -1920,
                    top: 0,
                    right: 0,
                    bottom: 1080,
                },
                1.0,
            ),
            (
                ScreenRect {
                    left: 0,
                    top: 0,
                    right: 2560,
                    bottom: 1440,
                },
                1.25,
            ),
        ];
        assert_eq!(
            monitor_of(&screens, 100, 100).map(|(rect, scale)| (rect.left, scale)),
            Some((0, 1.25))
        );
        // A second screen to the left has negative coordinates.
        assert_eq!(
            monitor_of(&screens, -100, 100).map(|(rect, scale)| (rect.left, scale)),
            Some((-1920, 1.0))
        );
        // A point outside every monitor, and the edge that belongs to the next
        // one: the right edge of a screen is the first pixel of the one after.
        assert!(monitor_of(&screens, 0, 2000).is_none());
        assert_eq!(
            monitor_of(&screens, 2560 - 1, 100).map(|(rect, _)| rect.left),
            Some(0)
        );
    }

    #[test]
    fn an_encoded_picture_is_a_png_that_keeps_its_size() {
        let full = shot(16, 8);
        let bytes = full.to_png(1024 * 1024).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        // The header carries the dimensions of the picture as big endian
        // integers after the magic number and the chunk length and name.
        assert_eq!(&bytes[16..20], &16u32.to_be_bytes());
        assert_eq!(&bytes[20..24], &8u32.to_be_bytes());
    }

    #[test]
    fn a_picture_over_the_limit_is_halved_instead_of_refused() {
        let full = shot(64, 64);
        let whole = full.to_png(usize::MAX).unwrap();
        let smaller = full.to_png(whole.len().saturating_sub(1)).unwrap();
        assert!(smaller.len() < whole.len());
        assert_eq!(&smaller[16..20], &32u32.to_be_bytes());
    }
}
