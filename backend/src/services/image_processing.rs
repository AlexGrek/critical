//! Image processing utilities for avatar and wallpaper uploads.
//!
//! Accepts JPEG, PNG, or WebP input; outputs two WebP variants (HD and thumbnail)
//! after center-cropping to the target aspect ratio and resizing.
//!
//! All logic is pure Rust — no C library wrappers (libvips, ImageMagick, etc.).

use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum accepted upload size (bytes). Checked before the image is stored.
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024; // 5 MB

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// What kind of image is being uploaded, controlling crop ratio and output sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadType {
    Avatar,
    Wallpaper,
}

impl UploadType {
    /// Object-store subdirectory for the processed files.
    pub fn storage_dir(self) -> &'static str {
        match self {
            UploadType::Avatar => "user_avatars",
            UploadType::Wallpaper => "user_wallpapers",
        }
    }

    /// Target crop aspect ratio (width : height).
    fn aspect(self) -> (u32, u32) {
        match self {
            UploadType::Avatar => (1, 1),
            UploadType::Wallpaper => (21, 9),
        }
    }

    /// HD output dimensions in pixels.
    fn hd_size(self) -> (u32, u32) {
        match self {
            UploadType::Avatar => (480, 480),
            UploadType::Wallpaper => (1400, 600),
        }
    }

    /// Thumbnail output dimensions in pixels.
    fn thumb_size(self) -> (u32, u32) {
        match self {
            UploadType::Avatar => (128, 128),
            UploadType::Wallpaper => (300, 128),
        }
    }
}

/// Recognised input image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageInputFormat {
    Jpeg,
    Png,
    Webp,
}

impl ImageInputFormat {
    /// Lowercase file extension without leading dot.
    pub fn extension(self) -> &'static str {
        match self {
            ImageInputFormat::Jpeg => "jpg",
            ImageInputFormat::Png => "png",
            ImageInputFormat::Webp => "webp",
        }
    }
}

/// Result of a successful image processing pass — two WebP byte buffers.
#[derive(Debug)]
pub struct ProcessedImages {
    pub hd: Bytes,
    pub thumb: Bytes,
    pub hd_size_bytes: u64,
    pub thumb_size_bytes: u64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("unsupported or unrecognized image format (accepted: jpeg, png, webp)")]
    UnsupportedFormat,
    #[error("image decode error: {0}")]
    Decode(#[from] image::ImageError),
    #[error("WebP encode error: {0}")]
    Encode(String),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect image format from magic bytes.
///
/// Returns `None` if the byte sequence does not match JPEG, PNG, or WebP.
/// This check happens before any I/O, so callers can reject invalid uploads
/// without storing anything.
pub fn detect_format(bytes: &[u8]) -> Option<ImageInputFormat> {
    if bytes.len() < 12 {
        return None;
    }
    // JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageInputFormat::Jpeg);
    }
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageInputFormat::Png);
    }
    // WebP: RIFF????WEBP
    if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ImageInputFormat::Webp);
    }
    None
}

/// Decode `raw`, center-crop to the target aspect ratio, produce HD + thumbnail
/// WebP outputs. Upscaling is allowed so small inputs always yield the target size.
///
/// Returns `ProcessingError::UnsupportedFormat` if magic bytes are not recognised.
pub fn process_image(
    raw: &[u8],
    upload_type: UploadType,
) -> Result<ProcessedImages, ProcessingError> {
    detect_format(raw).ok_or(ProcessingError::UnsupportedFormat)?;

    let img = image::load_from_memory(raw)?;
    let (ratio_w, ratio_h) = upload_type.aspect();
    let cropped = crop_to_aspect(img, ratio_w, ratio_h);

    let (hd_w, hd_h) = upload_type.hd_size();
    let (th_w, th_h) = upload_type.thumb_size();

    let hd_img = cropped.resize_exact(hd_w, hd_h, FilterType::Lanczos3);
    let thumb_img = cropped.resize_exact(th_w, th_h, FilterType::Lanczos3);

    let hd = encode_webp(&hd_img)?;
    let thumb = encode_webp(&thumb_img)?;

    let hd_size_bytes = hd.len() as u64;
    let thumb_size_bytes = thumb.len() as u64;

    Ok(ProcessedImages {
        hd,
        thumb,
        hd_size_bytes,
        thumb_size_bytes,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Center-crop `img` to the given aspect ratio (width:height).
///
/// If the image is wider than the target ratio, columns are cropped equally
/// from both sides. If it is taller, rows are cropped from top and bottom.
///
/// Uses integer cross-multiplication for the aspect comparison to avoid
/// floating-point rounding errors on exact ratios (e.g. a 2100×900 image
/// with a 21:9 target should produce exactly (2100, 900)).
fn crop_to_aspect(img: DynamicImage, ratio_w: u32, ratio_h: u32) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Compare width/height vs ratio_w/ratio_h without floating point:
    //   wider  ⟺  width * ratio_h > height * ratio_w
    let (crop_w, crop_h) = if width as u64 * ratio_h as u64 > height as u64 * ratio_w as u64 {
        // Wider than target: trim sides
        let cw = (height as u64 * ratio_w as u64 / ratio_h as u64) as u32;
        (cw.min(width), height)
    } else {
        // Taller (or equal): trim top/bottom
        let ch = (width as u64 * ratio_h as u64 / ratio_w as u64) as u32;
        (width, ch.min(height))
    };

    let x = (width - crop_w) / 2;
    let y = (height - crop_h) / 2;
    img.crop_imm(x, y, crop_w, crop_h)
}

/// Encode a `DynamicImage` to WebP bytes using the `image` crate's built-in encoder.
fn encode_webp(img: &DynamicImage) -> Result<Bytes, ProcessingError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::WebP)
        .map_err(|e| ProcessingError::Encode(e.to_string()))?;
    Ok(Bytes::from(buf.into_inner()))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Paths to real test assets in the itests directory. CARGO_MANIFEST_DIR points to
    // the `backend/` crate root, so itests/assets is a direct child.
    const ASSET_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/itests/assets");

    fn load_asset(filename: &str) -> Vec<u8> {
        let path = format!("{ASSET_DIR}/{filename}");
        std::fs::read(&path)
            .unwrap_or_else(|e| panic!("failed to read test asset {path}: {e}"))
    }

    /// Encode a blank in-memory image to the given format, returning raw bytes.
    /// Used to synthesize test inputs without needing a file on disk.
    fn make_image_bytes(w: u32, h: u32, fmt: image::ImageFormat) -> Vec<u8> {
        let img = DynamicImage::new_rgb8(w, h);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, fmt).unwrap();
        buf.into_inner()
    }

    // ── format detection ────────────────────────────────────────────────────

    #[test]
    fn detect_jpeg_from_file() {
        let data = load_asset("photo_2025-09-13_00-46-10.jpg");
        assert_eq!(detect_format(&data), Some(ImageInputFormat::Jpeg));
    }

    #[test]
    fn detect_webp_from_file() {
        let data = load_asset("audi_a5__622692711hd.webp");
        assert_eq!(detect_format(&data), Some(ImageInputFormat::Webp));
    }

    #[test]
    fn detect_png_synthesized() {
        let data = make_image_bytes(100, 100, image::ImageFormat::Png);
        assert_eq!(detect_format(&data), Some(ImageInputFormat::Png));
    }

    #[test]
    fn detect_jpeg_synthesized() {
        let data = make_image_bytes(100, 100, image::ImageFormat::Jpeg);
        assert_eq!(detect_format(&data), Some(ImageInputFormat::Jpeg));
    }

    #[test]
    fn detect_unknown_returns_none() {
        assert_eq!(detect_format(b"not an image"), None);
        assert_eq!(detect_format(b""), None);
        assert_eq!(detect_format(&[0u8; 4]), None);
    }

    // ── crop_to_aspect ──────────────────────────────────────────────────────

    #[test]
    fn crop_square_from_landscape() {
        // 200×100 landscape → 1:1 → should yield 100×100
        let img = DynamicImage::new_rgb8(200, 100);
        let cropped = crop_to_aspect(img, 1, 1);
        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn crop_square_from_portrait() {
        // 100×200 portrait → 1:1 → should yield 100×100
        let img = DynamicImage::new_rgb8(100, 200);
        let cropped = crop_to_aspect(img, 1, 1);
        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn crop_wide_from_tall() {
        // 630×400 image, 21:9 target ratio ≈ 2.333
        // actual ratio = 1.575 (taller) → trim height
        // crop_h = 630 / (21/9) = 270
        let img = DynamicImage::new_rgb8(630, 400);
        let cropped = crop_to_aspect(img, 21, 9);
        assert_eq!(cropped.dimensions(), (630, 270));
    }

    #[test]
    fn crop_wide_from_exact_ratio() {
        // 2100×900 is exactly 21:9 — no crop should occur
        let img = DynamicImage::new_rgb8(2100, 900);
        let cropped = crop_to_aspect(img, 21, 9);
        assert_eq!(cropped.dimensions(), (2100, 900));
    }

    #[test]
    fn crop_wide_from_very_wide() {
        // 4000×900 is wider than 21:9 — trim sides
        // crop_w = 900 * (21/9) = 2100
        let img = DynamicImage::new_rgb8(4000, 900);
        let cropped = crop_to_aspect(img, 21, 9);
        assert_eq!(cropped.dimensions(), (2100, 900));
    }

    // ── process_image with real assets ──────────────────────────────────────

    #[test]
    fn process_jpg_avatar() {
        let data = load_asset("photo_2025-09-13_00-46-10.jpg");
        let result = process_image(&data, UploadType::Avatar).expect("processing failed");

        assert_eq!(result.hd_size_bytes, result.hd.len() as u64);
        assert_eq!(result.thumb_size_bytes, result.thumb.len() as u64);

        let hd_img = image::load_from_memory(&result.hd).expect("hd not decodable");
        assert_eq!(hd_img.dimensions(), (480, 480));

        let thumb_img = image::load_from_memory(&result.thumb).expect("thumb not decodable");
        assert_eq!(thumb_img.dimensions(), (128, 128));
    }

    #[test]
    fn process_webp_wallpaper() {
        let data = load_asset("audi_a5__622692711hd.webp");
        let result = process_image(&data, UploadType::Wallpaper).expect("processing failed");

        let hd_img = image::load_from_memory(&result.hd).expect("hd not decodable");
        assert_eq!(hd_img.dimensions(), (1400, 600));

        let thumb_img = image::load_from_memory(&result.thumb).expect("thumb not decodable");
        assert_eq!(thumb_img.dimensions(), (300, 128));
    }

    #[test]
    fn process_png_avatar() {
        let data = make_image_bytes(400, 600, image::ImageFormat::Png);
        let result = process_image(&data, UploadType::Avatar).expect("processing failed");

        let hd_img = image::load_from_memory(&result.hd).expect("hd not decodable");
        assert_eq!(hd_img.dimensions(), (480, 480));

        let thumb_img = image::load_from_memory(&result.thumb).expect("thumb not decodable");
        assert_eq!(thumb_img.dimensions(), (128, 128));
    }

    #[test]
    fn process_rejects_garbage() {
        let err = process_image(b"not an image at all", UploadType::Avatar)
            .expect_err("should fail on garbage input");
        assert!(matches!(err, ProcessingError::UnsupportedFormat));
    }

    // ── object store integration (in-memory) ────────────────────────────────

    #[tokio::test]
    async fn store_and_retrieve_processed_images() {
        use std::sync::Arc;

        use object_store::memory::InMemory;

        use crate::services::objectstore::ObjectStoreService;

        let store = ObjectStoreService::from_store(Arc::new(InMemory::new()));

        let data = load_asset("photo_2025-09-13_00-46-10.jpg");
        let result = process_image(&data, UploadType::Avatar).expect("processing failed");

        let ulid = "01test00000000000000000000";
        let hd_path = format!("user_avatars/{ulid}_hd.webp");
        let thumb_path = format!("user_avatars/{ulid}_thumb.webp");

        store.put(&hd_path, result.hd.clone()).await.unwrap();
        store.put(&thumb_path, result.thumb.clone()).await.unwrap();

        // Retrieval returns identical bytes
        let fetched_hd = store.get(&hd_path).await.unwrap();
        assert_eq!(fetched_hd, result.hd);

        // Stored bytes must be valid WebP at the correct dimensions
        let decoded = image::load_from_memory(&fetched_hd).expect("stored hd is not valid webp");
        assert_eq!(decoded.dimensions(), (480, 480));

        // Prefix listing should find both files
        let listing = store.list("user_avatars").await.unwrap();
        assert_eq!(listing.len(), 2);

        // Deletion works
        store.delete(&hd_path).await.unwrap();
        store.delete(&thumb_path).await.unwrap();
        assert!(store.get(&hd_path).await.is_err());
    }
}
