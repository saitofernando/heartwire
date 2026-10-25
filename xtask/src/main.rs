//! Repository tasks: `cargo xtask icons`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use image::{
    ExtendedColorType, ImageEncoder, Rgba, RgbaImage,
    codecs::{
        ico::{IcoEncoder, IcoFrame},
        png::PngEncoder,
    },
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    match std::env::args().nth(1).as_deref() {
        Some("icons") => icons(&root()),
        _ => Err("Uso / Usage: cargo xtask icons".into()),
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

struct Cell {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
}

/// The approved pixel art is the source of truth; every platform asset is
/// sampled from its 16×16 grid and never redrawn.
fn icons(root: &Path) -> Result {
    let original = fs::read_to_string(root.join("design/heartwire/heartwire.svg"))?;
    let start = original.find("  <g").ok_or("heartwire.svg: <g> ausente")?;
    let end = original
        .rfind("</g>")
        .ok_or("heartwire.svg: </g> ausente")?
        + "</g>".len();
    let drawing = &original[start..end];
    let cells = parse(drawing)?;

    for file in ["web/icons/heartwire.svg", "docs/assets/heartwire.svg"] {
        fs::write(root.join(file), &original)?;
    }
    let framed = |grid: u32, offset: u32| {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"256\" height=\"256\" viewBox=\"0 0 {grid} {grid}\" shape-rendering=\"crispEdges\" role=\"img\" aria-labelledby=\"title\"><title id=\"title\">HeartWire</title><rect width=\"{grid}\" height=\"{grid}\" fill=\"#101615\"/><g transform=\"translate({offset} {offset})\">{drawing}</g></svg>\n"
        )
    };
    fs::write(root.join("design/heartwire/app.svg"), framed(20, 2))?;
    fs::write(root.join("design/heartwire/maskable.svg"), framed(24, 4))?;

    let mark = |size| render(&cells, 16, 0, None, size);
    let background = Some(Rgba([0x10, 0x16, 0x15, 0xff]));
    let app = |size| render(&cells, 20, 2, background, size);
    let desktop = root.join("crates/heartwire/icons");
    png(&mark(32), &desktop.join("32x32.png"))?;
    png(&mark(128), &desktop.join("128x128.png"))?;
    png(&mark(32), &desktop.join("tray.png"))?;
    let frames = [16, 24, 32, 48, 64, 256]
        .into_iter()
        .map(|size| {
            let image = mark(size);
            IcoFrame::as_png(image.as_raw(), size, size, ExtendedColorType::Rgba8)
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    IcoEncoder::new(fs::File::create(desktop.join("icon.ico"))?).encode_images(&frames)?;
    let web = root.join("web/icons");
    png(&app(180), &web.join("apple-touch-icon.png"))?;
    png(&app(192), &web.join("icon-192.png"))?;
    png(&app(512), &web.join("icon-512.png"))?;
    png(
        &render(&cells, 24, 4, background, 512),
        &web.join("maskable-512.png"),
    )?;
    println!("HeartWire: SVG colorido, ícone de aplicativo, PNG e ICO.");
    Ok(())
}

fn parse(drawing: &str) -> Result<Vec<Cell>> {
    let mut cells = Vec::new();
    let mut color = None;
    for tag in drawing.split('<').skip(1) {
        if tag.starts_with("g ") {
            color = Some(hex(attribute(tag, "fill").ok_or("<g> sem fill")?)?);
        } else if tag.starts_with("rect ") {
            let number = |name| -> Result<u32> {
                Ok(attribute(tag, name).ok_or("rect incompleto")?.parse()?)
            };
            cells.push(Cell {
                x: number("x")?,
                y: number("y")?,
                width: number("width")?,
                height: number("height")?,
                color: color.ok_or("rect fora de <g fill>")?,
            });
        }
    }
    Ok(cells)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn hex(value: &str) -> Result<Rgba<u8>> {
    let value = value.strip_prefix('#').ok_or("cor inválida")?;
    let channel = |index: usize| u8::from_str_radix(&value[index..index + 2], 16);
    Ok(Rgba([channel(0)?, channel(2)?, channel(4)?, 0xff]))
}

/// Nearest-cell sampling at pixel centres keeps the art crisp at any size.
fn render(
    cells: &[Cell],
    grid: u32,
    offset: u32,
    background: Option<Rgba<u8>>,
    size: u32,
) -> RgbaImage {
    let mut image = RgbaImage::from_pixel(size, size, background.unwrap_or(Rgba([0, 0, 0, 0])));
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let cell = |p: u32| ((p as f64 + 0.5) * grid as f64 / size as f64) as u32;
        let (cx, cy) = (cell(x), cell(y));
        if let Some(found) = cells.iter().rev().find(|c| {
            (c.x + offset..c.x + offset + c.width).contains(&cx)
                && (c.y + offset..c.y + offset + c.height).contains(&cy)
        }) {
            *pixel = found.color;
        }
    }
    image
}

fn png(image: &RgbaImage, path: &Path) -> Result {
    PngEncoder::new(fs::File::create(path)?).write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ExtendedColorType::Rgba8,
    )?;
    Ok(())
}
