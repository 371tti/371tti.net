pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlDecodeError {
    InvalidPercentEncoding { index: usize },
    InvalidUtf8,
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// application/x-www-form-urlencoded 風:
/// - 英数字と `- . _ ~` はそのまま
/// - 空白は `+`
/// - それ以外は `%XX`
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

/// `url_encode` の逆変換:
/// - `+` は空白
/// - `%XX` をデコード
pub fn url_decode(s: &str) -> Result<String, UrlDecodeError> {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(UrlDecodeError::InvalidPercentEncoding { index: i });
                }
                let hi = hex_val(bytes[i + 1]).ok_or(UrlDecodeError::InvalidPercentEncoding { index: i })?;
                let lo = hex_val(bytes[i + 2]).ok_or(UrlDecodeError::InvalidPercentEncoding { index: i })?;
                out.push((hi << 4) | lo);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8(out).map_err(|_| UrlDecodeError::InvalidUtf8)
}