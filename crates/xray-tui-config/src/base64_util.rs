use std::borrow::Cow;

/// Decode base64 with trailing annotation stripping and URL-safe/Standard fallback.
///
/// Stray backtick characters filtered, then percent-decoded (handles %XX in base64
/// content), trailing non-base64 annotation text stripped, tries
/// `BASE64_URL_SAFE_NO_PAD` first then `BASE64_STANDARD_NO_PAD`.
///
/// Stray backtick characters filtered, then percent-decoded (handles %XX in base64
/// content), trailing non-base64 annotation text stripped, tries
/// `URL_SAFE_NO_PAD` first then `STANDARD_NO_PAD`.
///
/// # Errors
///
/// Returns a `base64_simd::Error` if neither encoding produces valid output,
/// or if the input is empty after cleaning.
pub fn decode_base64(data: &str) -> Result<Vec<u8>, base64_simd::Error> {
    // Strip stray backtick characters
    let cleaned: String = data.chars().filter(|&c| c != '`').collect();

    // Percent-decode first (handles %XX sequences that may represent base64 chars)
    let decoded = simple_percent_decode(&cleaned);

    // Find end of valid base64 characters in the decoded string
    let end = decoded
        .as_ref()
        .find(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '+' | '/' | '-' | '_' | '='))
        .unwrap_or_else(|| decoded.as_ref().len());
    let mut data = &decoded.as_ref()[..end];

    // After the last padding marker (`==` or `=`), strip trailing annotation text.
    if let Some(pos) = data.rfind("==") {
        data = &data[..pos + 2];
    } else if let Some(pos) = data.rfind('=') {
        data = &data[..=pos];
    }

    let trimmed = data.trim_end_matches(|c: char| c == '=' || c.is_whitespace());

    if trimmed.is_empty() {
        return Err(base64_simd::STANDARD_NO_PAD
            .decode_to_vec(b"!")
            .unwrap_err());
    }

    'block: {
        let e = match base64_simd::URL_SAFE_NO_PAD.decode_to_vec(trimmed.as_bytes()) {
            Ok(r) => break 'block Ok(r),
            Err(e) => e,
        };
        if let Ok(r) = base64_simd::STANDARD_NO_PAD.decode_to_vec(trimmed.as_bytes()) {
            break 'block Ok(r);
        }
        Err(e)
    }
}

/// Percent-decode a string, replacing %XX sequences with decoded bytes.
fn simple_percent_decode(s: &str) -> Cow<'_, str> {
    urlencoding::decode(s).map_or(Cow::Borrowed(s), |d| Cow::Owned(d.into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_base64() {
        let result = decode_base64("aGVsbG8=").unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_url_safe_base64() {
        let result = decode_base64("aGVsbG8").unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_trailing_annotation() {
        let result = decode_base64("aGVsbG8=Irancell").unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_backtick_filtering() {
        let result = decode_base64("aGVs`bG8=").unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_url_safe_no_pad() {
        let result = decode_base64("Pj4-Pj4-Pj4").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_invalid_base64() {
        assert!(decode_base64("!!!not-base64!!!").is_err());
    }

    #[test]
    fn test_empty_input() {
        assert!(decode_base64("").is_err());
    }

    #[test]
    fn test_percent_encoded_chars() {
        // %2F = / (standard base64 valid), %2B = + (standard base64 valid)
        // After percent-decoding: "aGVs/G8+" which contains standard base64 chars
        let result = decode_base64("aGVs%2FG8%2B").unwrap();
        // 011010 000110 010101 111111 000110 111100 111110 (with empty pad)
        // aGVs/G8+ in standard = 3 bytes + 3 bytes = 6 bytes
        assert_eq!(result.len(), 6);
        assert!(result.starts_with(b"hel"));
    }
}
