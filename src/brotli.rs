// SPDX-License-Identifier: MIT
//!
//! Brotli ransparent compression
//! Supports
//!   Content-Encoding: br
//!

///
/// Trait to check if reponse should be compressed
///
pub(crate) trait ResponseCompression {
    /// Content-Encoding header value
    fn content_encoding<'a>(&'a self) -> Option<&'a str>;

    /// Content-Type header value
    fn content_type<'a>(&'a self) -> Option<&'a str>;

    /// Can this response be compressed?
    #[cfg(feature = "br")]
    fn can_brotli_compress(&self) -> bool {
        // Check already compressed
        if self.content_encoding().is_some() {
            // Already compressed
            return false;
        }

        // Get Content-type header value
        if let Some(header_val) = self.content_type() {
            let ctype = header_val.trim().to_ascii_lowercase();

            // Compress when text types
            ctype.starts_with("text/")
                || ctype.starts_with("application/json")
                || ctype.starts_with("application/xhtml")
                || ctype.starts_with("application/xml")
                || ctype.starts_with("application/wasm")
                || ctype.starts_with("image/svg")
        } else {
            // No content-type
            false
        }
    }

    // Without Brotli support, always returns false
    #[cfg(not(feature = "br"))]
    fn can_brotli_compress(&self) -> bool {
        false
    }
}

/// Compress response using Brotli, base64 encode it, and return encoded string.
#[cfg(feature = "br")]
pub(crate) fn compress_response_body<'a>(body: &[u8]) -> String {
    // Compress parameter
    let cfg = brotli::enc::BrotliEncoderParams {
        quality: 4,
        ..Default::default()
    };

    // Do Brotli compression
    let mut body_reader = std::io::Cursor::new(body);
    let mut compressed_base64 = base64::write::EncoderStringWriter::new(base64::STANDARD);
    let _sz = brotli::BrotliCompress(&mut body_reader, &mut compressed_base64, &cfg);

    compressed_base64.into_inner()
}

// No Brotli compression, only base64 encoding
#[cfg(not(feature = "br"))]
pub(crate) fn compress_response_body<'a>(body: &[u8]) -> String {
    base64::encode(body)
}
