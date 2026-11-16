//! The head of a picture frame.
//!
//! What: the reading of the first bytes of an `APIC` frame — the type of the
//!       picture, the name of its file and the place where the picture starts.
//! Why: the layout of the frame depends on the version of the tag and on the
//!       encoding of the text, and a frame that lies about its layout must
//!       give no picture instead of a picture of the wrong bytes (`F-10`).
//! Example: `PictureHead::parse` says where the picture of a frame begins.

pub(super) struct PictureHead {
    /// The type of the picture.
    pub(super) mime_type: String,
    /// The offset of the picture inside the frame.
    pub(super) offset: usize,
}

impl PictureHead {
    /// Parses the head of a picture frame.
    pub(super) fn parse(data: &[u8], version: u8) -> Option<Self> {
        let encoding = *data.first()?;
        let mut pos = 1;
        let mime_type = if version == 2 {
            // Version 2.2 names the format of the picture in three bytes.
            let format = data.get(pos..pos + 3)?;
            pos += 3;
            match format {
                b"JPG" => "image/jpeg".to_owned(),
                b"PNG" => "image/png".to_owned(),
                other => format!("image/{}", String::from_utf8_lossy(other).to_lowercase()),
            }
        } else {
            // The type of the picture is a text of ISO-8859-1 that ends with
            // a zero byte.
            let end = pos + data.get(pos..)?.iter().position(|byte| *byte == 0)?;
            let mime = data[pos..end]
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>();
            pos = end + 1;
            normalise_mime(&mime)?
        };
        // The picture type follows; it says where the picture is used and is
        // not needed here.
        pos += 1;
        if pos > data.len() {
            return None;
        }
        let offset = description_end(data, pos, encoding)?;
        Some(PictureHead { mime_type, offset })
    }
}

/// The offset of the picture, behind the description of a picture frame.
///
/// The description ends with a zero byte, or with two zero bytes when it is
/// written in UTF-16.
fn description_end(data: &[u8], start: usize, encoding: u8) -> Option<usize> {
    if encoding == 1 || encoding == 2 {
        return (start..data.len().saturating_sub(1))
            .step_by(2)
            .find(|index| data[*index] == 0 && data[*index + 1] == 0)
            .map(|index| index + 2);
    }
    if encoding > 3 {
        return None;
    }
    // ISO-8859-1 and UTF-8 end the description with one zero byte.
    data.get(start..)?
        .iter()
        .position(|byte| *byte == 0)
        .map(|end| start + end + 1)
}

/// The type of a picture, as the tag names it.
fn normalise_mime(mime: &str) -> Option<String> {
    let mime = mime.trim();
    match mime.to_ascii_lowercase().as_str() {
        "" => None,
        "jpg" | "jpeg" | "image/jpg" | "image/jpeg" => Some("image/jpeg".to_owned()),
        "png" | "image/png" => Some("image/png".to_owned()),
        _ => Some(mime.to_owned()),
    }
}
