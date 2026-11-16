//! The thumbnail of a photo.
//!
//! What: the production of a small JPEG out of a picture of the library.
//! Why: a control point asks for `upnp:albumArtURI` and expects a small
//!      picture, while a photo of a modern camera is far too large to be sent
//!      to a television. The production is therefore bounded twice: the size
//!      the file claims is checked before the pixels are decoded, and the
//!      pixels themselves never exceed the limits (`SEC-067`).
//! Example: `thumbnail` turns the bytes of a photo into the bytes of a JPEG
//!       whose longest edge is at most `limits.thumbnail_edge`.

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageReader};

use crate::media::artwork::PictureLimits;

/// The quality of the produced JPEG.
///
/// A quality of 80 keeps the artefacts of a thumbnail out of sight for a
/// television, and keeps the answer small enough to be sent over a network.
const QUALITY: u8 = 80;

/// The reason why a picture could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureError {
    /// The reason, in plain language, for the log and for the client.
    pub message: String,
}

impl PictureError {
    /// An error with the given reason.
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PictureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PictureError {}

/// Produces a thumbnail of a picture.
///
/// The declared size of the picture is checked before the picture is decoded,
/// so that a file that lies about its size cannot make the server allocate
/// without a bound. A picture that is smaller than the edge is not enlarged.
pub fn thumbnail(bytes: &[u8], limits: &PictureLimits) -> Result<Vec<u8>, PictureError> {
    if bytes.is_empty() {
        return Err(PictureError::new("the file is empty and holds no picture"));
    }
    let (width, height) = dimensions(bytes)?;
    refuse_when_too_large(width, height, limits)?;
    let picture = decode(bytes)?;
    let small = scale(picture, limits.thumbnail_edge)?;
    encode(small)
}

/// The size the picture claims to have.
///
/// Only the header of the file is read: the pixels are neither decoded nor
/// allocated yet, and that is the whole point of the check that follows
/// (`SEC-067`).
fn dimensions(bytes: &[u8]) -> Result<(u32, u32), PictureError> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| PictureError::new("the file could not be read as an image"))?
        .into_dimensions()
        .map_err(|_| PictureError::new("the file could not be read as an image"))
}

/// Refuses a picture that is larger than the limits allow.
///
/// The size the file claims is enough for the decision: a file that claims
/// more pixels than allowed is refused before a single pixel is allocated.
fn refuse_when_too_large(
    width: u32,
    height: u32,
    limits: &PictureLimits,
) -> Result<(), PictureError> {
    if width > limits.max_dimension || height > limits.max_dimension {
        return Err(PictureError::new(format!(
            "the picture is {width}x{height} pixels, longer than the limit of {} pixels",
            limits.max_dimension
        )));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > limits.max_pixels {
        return Err(PictureError::new(format!(
            "the picture has {pixels} pixels, more than the limit of {} pixels",
            limits.max_pixels
        )));
    }
    Ok(())
}

/// The picture itself, with its pixels.
fn decode(bytes: &[u8]) -> Result<DynamicImage, PictureError> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| PictureError::new("the file could not be read as an image"))?
        .decode()
        .map_err(|_| PictureError::new("the file could not be read as an image"))
}

/// The picture, made small enough to be sent to a television.
///
/// A picture that already fits is left alone: a thumbnail that is larger than
/// its source is not a thumbnail, and enlarging it would only waste bytes.
fn scale(picture: DynamicImage, edge: u32) -> Result<DynamicImage, PictureError> {
    if edge == 0 {
        return Err(PictureError::new(
            "the limit for the edge of a thumbnail is zero",
        ));
    }
    let longest = picture.width().max(picture.height());
    if longest <= edge {
        return Ok(picture);
    }
    let ratio = f64::from(edge) / f64::from(longest);
    let width = ((f64::from(picture.width()) * ratio).round() as u32).max(1);
    let height = ((f64::from(picture.height()) * ratio).round() as u32).max(1);
    Ok(picture.thumbnail_exact(width, height))
}

/// The picture, written as a JPEG.
///
/// JPEG has no alpha channel, so the picture is turned into RGB before it is
/// written: a picture with transparency would otherwise lose its alpha
/// channel in a way that depends on the encoder.
fn encode(picture: DynamicImage) -> Result<Vec<u8>, PictureError> {
    let rgb = picture.to_rgb8();
    let mut bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut bytes, QUALITY)
        .encode_image(&rgb)
        .map_err(|_| PictureError::new("the picture could not be written as a JPEG"))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{thumbnail, PictureError};
    use crate::media::artwork::PictureLimits;
    use image::codecs::jpeg::JpegEncoder;
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder, Rgb, RgbImage};
    use std::io::Cursor;

    /// The limits of a normal picture: big enough for a photo, small enough
    /// for a thumbnail.
    fn limits() -> PictureLimits {
        PictureLimits {
            max_embedded_bytes: 1024 * 1024,
            max_sidecar_bytes: 1024 * 1024,
            max_dimension: 1024,
            max_pixels: 4_000_000,
            thumbnail_edge: 64,
        }
    }

    /// A JPEG of the given size, filled with one colour.
    fn jpeg(width: u32, height: u32) -> Vec<u8> {
        let picture = RgbImage::from_pixel(width, height, Rgb([30, 60, 90]));
        let mut bytes = Vec::new();
        JpegEncoder::new_with_quality(&mut bytes, 90)
            .encode_image(&picture)
            .expect("a test picture can be written");
        bytes
    }

    /// The size of a produced picture.
    fn size_of(bytes: &[u8]) -> (u32, u32) {
        image::ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .expect("the produced picture is a JPEG")
            .into_dimensions()
            .expect("the produced picture has a size")
    }

    /// A PNG that claims to be a huge picture.
    ///
    /// The picture itself stays small: only the header lies. A decoder that
    /// asks for the size is told the size of a huge picture, so the check
    /// that follows refuses the file before a single pixel is decoded
    /// (`SEC-067`).
    fn lying_png(width: u32, height: u32) -> Vec<u8> {
        let small = RgbImage::from_pixel(4, 4, Rgb([1, 2, 3]));
        let mut cursor = Cursor::new(Vec::new());
        PngEncoder::new(&mut cursor)
            .write_image(small.as_raw(), 4, 4, ExtendedColorType::Rgb8)
            .expect("a test picture can be written");
        let mut bytes = cursor.into_inner();
        // The header of a PNG: the signature, the length of the chunk, the
        // type of the chunk, the size of the picture and the checksum.
        bytes[16..20].copy_from_slice(&width.to_be_bytes());
        bytes[20..24].copy_from_slice(&height.to_be_bytes());
        let checksum = crc32(&bytes[12..29]);
        bytes[29..33].copy_from_slice(&checksum.to_be_bytes());
        bytes
    }

    /// The checksum of the chunks of a PNG.
    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for byte in bytes {
            crc = step(crc ^ u32::from(*byte));
        }
        !crc
    }

    /// One step of the checksum: eight shifts of one bit.
    fn step(crc: u32) -> u32 {
        let mut shifted = crc;
        for _ in 0..8 {
            let mask = if shifted & 1 == 1 { 0xedb8_8320 } else { 0 };
            shifted = (shifted >> 1) ^ mask;
        }
        shifted
    }

    #[test]
    fn a_photo_is_scaled_down_to_the_edge() {
        let produced = thumbnail(&jpeg(200, 100), &limits()).expect("the photo is scaled");
        let (width, height) = size_of(&produced);
        assert_eq!(width, 64);
        assert_eq!(height, 32);
    }

    #[test]
    fn a_small_photo_is_not_enlarged() {
        let produced = thumbnail(&jpeg(20, 10), &limits()).expect("the photo is kept");
        let (width, height) = size_of(&produced);
        assert_eq!(width, 20);
        assert_eq!(height, 10);
    }

    #[test]
    fn a_picture_that_lies_about_its_size_is_refused_before_it_is_decoded() {
        let error = thumbnail(&lying_png(5000, 5000), &limits())
            .expect_err("a picture that lies about its size is refused");
        assert!(
            error.message.contains("limit"),
            "the reason is not about the limit: {}",
            error.message
        );
    }

    #[test]
    fn a_picture_with_too_many_pixels_is_refused() {
        let mut small = limits();
        small.max_pixels = 100;
        let error = thumbnail(&jpeg(40, 40), &small).expect_err("too many pixels are refused");
        assert!(
            error.message.contains("more than the limit"),
            "the reason is not about the limit: {}",
            error.message
        );
    }

    #[test]
    fn a_file_that_is_not_an_image_is_refused() {
        let error = thumbnail(b"this is a text file, not a picture", &limits())
            .expect_err("a text file is refused");
        assert!(error.message.contains("image"), "{}", error.message);
    }

    #[test]
    fn an_empty_file_is_refused() {
        let error = thumbnail(&[], &limits()).expect_err("an empty file is refused");
        assert!(error.message.contains("empty"), "{}", error.message);
    }

    #[test]
    fn the_produced_picture_is_a_jpeg() {
        let produced = thumbnail(&jpeg(200, 100), &limits()).expect("the photo is scaled");
        assert_eq!(&produced[0..2], &[0xff, 0xd8], "the picture is not a JPEG");
    }

    #[test]
    fn the_reason_is_readable() {
        let error = PictureError {
            message: "the file could not be read as an image".to_owned(),
        };
        assert_eq!(error.to_string(), "the file could not be read as an image");
    }
}
