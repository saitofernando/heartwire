use eframe::egui::{Color32, ColorImage};
use qrcode::{Color, EcLevel, QrCode};

// Encode locally: the phone address never goes to an external QR service.
// Keep a four-module quiet zone and high recovery for the small centre mark.
const QUIET: usize = 4;
const DARK: Color32 = Color32::from_rgb(0x12, 0x21, 0x1c);
const MARK_BACKGROUND: Color32 = Color32::from_rgb(0x17, 0x1b, 0x18);

/// Pixel art for the centre mark, decoded once from the app icon.
pub struct Mark {
    size: usize,
    rgba: Vec<u8>,
}

impl Mark {
    pub fn decode(png: &[u8]) -> Option<Self> {
        let image = image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .ok()?
            .into_rgba8();
        (image.width() == image.height()).then(|| Self {
            size: image.width() as usize,
            rgba: image.into_raw(),
        })
    }
}

/// Renders whole modules of at least `min_pixels` per side, so the texture can
/// be drawn with nearest filtering and stay sharp.
pub fn pairing_qr(url: &str, min_pixels: usize, mark: Option<&Mark>) -> Result<ColorImage, String> {
    let code = QrCode::with_error_correction_level(url, EcLevel::H).map_err(|e| e.to_string())?;
    let modules = code.width();
    let total = modules + QUIET * 2;
    let scale = min_pixels.div_ceil(total).max(1);
    let size = total * scale;
    let mut image = ColorImage::filled([size, size], Color32::WHITE);
    for (index, color) in code.to_colors().into_iter().enumerate() {
        if color != Color::Dark {
            continue;
        }
        let (x0, y0) = (
            (index % modules + QUIET) * scale,
            (index / modules + QUIET) * scale,
        );
        for y in y0..y0 + scale {
            image.pixels[y * size + x0..y * size + x0 + scale].fill(DARK);
        }
    }
    if let Some(mark) = mark {
        // The complete mark covers 18.75% of the side; the white rim separates
        // it from data modules and leaves finder patterns untouched.
        let tile = size * 3 / 16;
        let inset = size / 64;
        let artwork = size * 9 / 64;
        fill(&mut image, (size - tile) / 2, tile, Color32::WHITE);
        fill(
            &mut image,
            (size - tile) / 2 + inset,
            tile - inset * 2,
            MARK_BACKGROUND,
        );
        let origin = (size - artwork) / 2;
        for y in 0..artwork {
            for x in 0..artwork {
                let source = ((y * mark.size / artwork) * mark.size + x * mark.size / artwork) * 4;
                let [r, g, b, a] = mark.rgba[source..source + 4] else {
                    continue;
                };
                if a > 0 {
                    let pixel = &mut image.pixels[(origin + y) * size + origin + x];
                    *pixel = blend(*pixel, Color32::from_rgb(r, g, b), a);
                }
            }
        }
    }
    Ok(image)
}

fn fill(image: &mut ColorImage, start: usize, side: usize, color: Color32) {
    let size = image.size[0];
    for y in start..start + side {
        image.pixels[y * size + start..y * size + start + side].fill(color);
    }
}

fn blend(under: Color32, over: Color32, alpha: u8) -> Color32 {
    let mix =
        |a: u8, b: u8| ((a as u16 * (255 - alpha as u16) + b as u16 * alpha as u16) / 255) as u8;
    Color32::from_rgb(
        mix(under.r(), over.r()),
        mix(under.g(), over.g()),
        mix(under.b(), over.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://192.168.0.10:8081/?mode=listen&lang=pt-BR";

    fn decode(image: &ColorImage) -> String {
        let [width, height] = image.size;
        let luma: Vec<u8> = image.pixels.iter().map(|pixel| pixel.g()).collect();
        let mut prepared =
            rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| luma[y * width + x]);
        let grids = prepared.detect_grids();
        assert_eq!(grids.len(), 1);
        grids[0].decode().unwrap().1
    }

    #[test]
    fn modules_are_whole_pixels_with_quiet_zone() {
        let image = pairing_qr(URL, 192, None).unwrap();
        let [width, height] = image.size;
        assert_eq!(width, height);
        assert!(width >= 192);
        assert_eq!(image.pixels[0], Color32::WHITE);
        // The top-left finder pattern starts right after the quiet zone.
        let first_dark = image
            .pixels
            .iter()
            .position(|pixel| *pixel == DARK)
            .unwrap();
        assert_eq!(first_dark % width, first_dark / width);
        assert_eq!(decode(&image), URL);
    }

    #[test]
    fn centre_mark_keeps_the_code_readable() {
        let mark = Mark::decode(include_bytes!("../icons/128x128.png")).unwrap();
        let image = pairing_qr(URL, 256, Some(&mark)).unwrap();
        let size = image.size[0];
        let corner = (size - size * 3 / 16) / 2;
        assert_eq!(image.pixels[corner * size + corner], Color32::WHITE);
        let inner = corner + size / 64;
        assert_eq!(image.pixels[inner * size + inner], MARK_BACKGROUND);
        assert_eq!(decode(&image), URL);
    }
}
