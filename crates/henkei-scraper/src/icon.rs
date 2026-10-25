use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use std::io::{Cursor, Write};
use std::path::Path;
use url::Url;

#[derive(Debug, Clone)]
pub struct ScrapedIconCandidate {
    pub url: Url,
    pub size_score: u32,
    pub rel: String,
}

pub fn parse_sizes_to_score(sizes_opt: Option<&str>, rel: &str) -> u32 {
    let mut score = 32;

    if rel == "apple-touch-icon" {
        score = 180;
    } else if rel == "og:image" {
        score = 120;
    }

    if let Some(sizes) = sizes_opt {
        let trimmed = sizes.trim();
        if trimmed.eq_ignore_ascii_case("any") {
            return 1024;
        }

        for part in trimmed.split_whitespace() {
            if let Some((w, _)) = part.split_once('x') {
                if let Ok(val) = w.parse::<u32>() {
                    if val > score {
                        score = val;
                    }
                }
            }
        }
    }

    score
}

pub fn download_and_decode_icon(
    client: &reqwest::blocking::Client,
    url: &Url,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let res = client.get(url.as_str()).send()?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()).into());
    }

    let bytes = res.bytes()?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

pub fn save_as_ico(img: &DynamicImage, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Generate high quality 256x256 RGBA for Windows
    let rgba = img.to_rgba8();
    let resized = image::imageops::resize(&rgba, 256, 256, FilterType::Lanczos3);
    resized.save_with_format(output_path, ImageFormat::Ico)?;
    Ok(())
}

pub fn save_as_png(
    img: &DynamicImage,
    output_path: &Path,
    size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rgba = img.to_rgba8();
    let resized = image::imageops::resize(&rgba, size, size, FilterType::Lanczos3);
    resized.save_with_format(output_path, ImageFormat::Png)?;
    Ok(())
}

/// Generates a valid Apple ICNS file containing modern PNG-encoded icons
pub fn save_as_icns(img: &DynamicImage, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let targets = [
        (b"icp4", 16),
        (b"icp5", 32),
        (b"icp6", 64),
        (b"ic07", 128),
        (b"ic08", 256),
        (b"ic09", 512),
        (b"ic10", 1024),
    ];

    let mut body = Vec::new();
    let rgba = img.to_rgba8();

    for (tag, size) in targets {
        let resized = image::imageops::resize(&rgba, size, size, FilterType::Lanczos3);
        let mut png_bytes = Vec::new();
        resized.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;

        // Each entry: 4 bytes tag + 4 bytes total entry size (big endian) + PNG bytes
        let entry_size = (8 + png_bytes.len()) as u32;
        body.write_all(tag)?;
        body.write_all(&entry_size.to_be_bytes())?;
        body.write_all(&png_bytes)?;
    }

    // Header: b"icns" + total file length (8 + body.len())
    let total_file_size = (8 + body.len()) as u32;
    let mut file_bytes = Vec::with_capacity(total_file_size as usize);
    file_bytes.write_all(b"icns")?;
    file_bytes.write_all(&total_file_size.to_be_bytes())?;
    file_bytes.write_all(&body)?;

    std::fs::write(output_path, file_bytes)?;
    Ok(())
}
