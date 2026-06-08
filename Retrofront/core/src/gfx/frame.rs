use crate::libretro;
use std::os::raw::c_void;

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

/// Libretro software pixel formats accepted by `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    ZeroRgb1555,
    Xrgb8888,
    Rgb565,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self::ZeroRgb1555
    }
}

impl PixelFormat {
    pub fn from_libretro(value: u32) -> Option<Self> {
        match value {
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_0RGB1555 => {
                Some(Self::ZeroRgb1555)
            }
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_XRGB8888 => {
                Some(Self::Xrgb8888)
            }
            value if value == libretro::retro_pixel_format_RETRO_PIXEL_FORMAT_RGB565 => {
                Some(Self::Rgb565)
            }
            _ => None,
        }
    }

    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::ZeroRgb1555 | Self::Rgb565 => 2,
            Self::Xrgb8888 => 4,
        }
    }

    pub fn code(self) -> u32 {
        match self {
            Self::ZeroRgb1555 => 0,
            Self::Xrgb8888 => 1,
            Self::Rgb565 => 2,
        }
    }
}

/// Latest frame normalized for direct upload/display by platform code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub source_format: PixelFormat,
    /// Tight RGBA8888, top-left origin, one row after another.
    pub rgba: Vec<u8>,
}

impl Default for VideoFrame {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            pitch: 0,
            source_format: PixelFormat::default(),
            rgba: Vec::new(),
        }
    }
}

pub fn convert_frame_to_rgba(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
    pixel_format: PixelFormat,
) -> Result<VideoFrame, String> {
    if width == 0 || height == 0 {
        return Ok(VideoFrame {
            width,
            height,
            pitch,
            source_format: pixel_format,
            rgba: Vec::new(),
        });
    }
    if data.is_null() || data == RETRO_HW_FRAME_BUFFER_VALID {
        return Err("hardware frames require a GL/Vulkan driver path before display".into());
    }
    let min_pitch = width as usize * pixel_format.bytes_per_pixel();
    if pitch < min_pitch {
        return Err(format!(
            "frame pitch {pitch} is smaller than required {min_pitch}"
        ));
    }
    let byte_len = pitch
        .checked_mul(height as usize)
        .ok_or("frame dimensions overflow")?;
    let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), byte_len) };
    let mut rgba = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        let row = &bytes[y * pitch..y * pitch + min_pitch];
        for x in 0..width as usize {
            let out = (y * width as usize + x) * 4;
            match pixel_format {
                PixelFormat::Xrgb8888 => {
                    let p = u32::from_ne_bytes([
                        row[x * 4],
                        row[x * 4 + 1],
                        row[x * 4 + 2],
                        row[x * 4 + 3],
                    ]);
                    rgba[out] = ((p >> 16) & 0xff) as u8;
                    rgba[out + 1] = ((p >> 8) & 0xff) as u8;
                    rgba[out + 2] = (p & 0xff) as u8;
                    rgba[out + 3] = 0xff;
                }
                PixelFormat::Rgb565 => {
                    let p = u16::from_ne_bytes([row[x * 2], row[x * 2 + 1]]);
                    rgba[out] = expand_5(((p >> 11) & 0x1f) as u8);
                    rgba[out + 1] = expand_6(((p >> 5) & 0x3f) as u8);
                    rgba[out + 2] = expand_5((p & 0x1f) as u8);
                    rgba[out + 3] = 0xff;
                }
                PixelFormat::ZeroRgb1555 => {
                    let p = u16::from_ne_bytes([row[x * 2], row[x * 2 + 1]]);
                    rgba[out] = expand_5(((p >> 10) & 0x1f) as u8);
                    rgba[out + 1] = expand_5(((p >> 5) & 0x1f) as u8);
                    rgba[out + 2] = expand_5((p & 0x1f) as u8);
                    rgba[out + 3] = 0xff;
                }
            }
        }
    }
    Ok(VideoFrame {
        width,
        height,
        pitch,
        source_format: pixel_format,
        rgba,
    })
}

fn expand_5(v: u8) -> u8 {
    (v << 3) | (v >> 2)
}
fn expand_6(v: u8) -> u8 {
    (v << 2) | (v >> 4)
}
