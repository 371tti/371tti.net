use std::ops::Range;


pub struct Html2Text;

impl Html2Text {
    pub fn strip_html_text(input: &str) -> String {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Text,
            Tag,
            Script,
            Style,
        }

        let bytes = input.as_bytes();
        let mut out = String::with_capacity(input.len());
        let mut i = 0;
        let mut state = State::Text;
        let mut prev_was_space = false;

        while i < bytes.len() {
            match state {
                State::Text => {
                    if bytes[i] == b'<' {
                        if Self::starts_tag(bytes, i, b"script") {
                            state = State::Script;
                            i = Self::skip_tag(bytes, i);
                        } else if Self::starts_tag(bytes, i, b"style") {
                            state = State::Style;
                            i = Self::skip_tag(bytes, i);
                        } else {
                            state = State::Tag;
                            i += 1;
                        }
                    } else {
                        let ch = input[i..].chars().next().unwrap();
                        i += ch.len_utf8();

                        if ch.is_whitespace() {
                            if !prev_was_space {
                                out.push(' ');
                                prev_was_space = true;
                            }
                        } else {
                            out.push(ch);
                            prev_was_space = false;
                        }
                    }
                }

                State::Tag => {
                    if bytes[i] == b'>' {
                        state = State::Text;
                    }
                    i += 1;
                }

                State::Script => {
                    if Self::starts_end_tag(bytes, i, b"script") {
                        i = Self::skip_tag(bytes, i);
                        state = State::Text;
                    } else {
                        i += 1;
                    }
                }

                State::Style => {
                    if Self::starts_end_tag(bytes, i, b"style") {
                        i = Self::skip_tag(bytes, i);
                        state = State::Text;
                    } else {
                        i += 1;
                    }
                }
            }
        }

        out.trim().to_string()
    }

    fn skip_tag(bytes: &[u8], mut i: usize) -> usize {
        while i < bytes.len() && bytes[i] != b'>' {
            i += 1;
        }
        if i < bytes.len() {
            i += 1;
        }
        i
    }

    fn starts_tag(bytes: &[u8], i: usize, tag: &[u8]) -> bool {
        if i >= bytes.len() || bytes[i] != b'<' {
            return false;
        }

        let mut j = i + 1;

        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }

        Self::starts_ascii_case_insensitive(bytes, j, tag)
            && Self::after_tag_name_is_valid(bytes, j + tag.len())
    }

    fn starts_end_tag(bytes: &[u8], i: usize, tag: &[u8]) -> bool {
        if i + 1 >= bytes.len() || bytes[i] != b'<' || bytes[i + 1] != b'/' {
            return false;
        }

        let mut j = i + 2;

        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }

        Self::starts_ascii_case_insensitive(bytes, j, tag)
            && Self::after_tag_name_is_valid(bytes, j + tag.len())
    }

    fn starts_ascii_case_insensitive(bytes: &[u8], i: usize, pat: &[u8]) -> bool {
        if i + pat.len() > bytes.len() {
            return false;
        }

        bytes[i..i + pat.len()]
            .iter()
            .zip(pat.iter())
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
    }

    fn after_tag_name_is_valid(bytes: &[u8], i: usize) -> bool {
        if i >= bytes.len() {
            return true;
        }

        bytes[i].is_ascii_whitespace() || bytes[i] == b'>' || bytes[i] == b'/'
    }
}

pub fn parse_http_like_range(input: &str, len: usize) -> Option<Range<usize>> {
    let s = input.trim();

    if s.is_empty() {
        return None;
    }

    // 独自拡張: "10" => 10..30
    if !s.contains('-') {
        let start = s.parse::<usize>().ok()?;
        let start = start.min(len);
        let end = start.saturating_add(20).min(len);
        return Some(start..end);
    }

    let (lhs, rhs) = s.split_once('-')?;
    let lhs = lhs.trim();
    let rhs = rhs.trim();

    match (lhs.is_empty(), rhs.is_empty()) {
        // "-" は不正
        (true, true) => None,

        // "-N" => 末尾から N 個
        (true, false) => {
            let count = rhs.parse::<usize>().ok()?;
            if count == 0 {
                return None;
            }
            let start = len.saturating_sub(count);
            Some(start..len)
        }

        // "N-" => N から末尾まで
        (false, true) => {
            let start = lhs.parse::<usize>().ok()?;
            if start >= len {
                return None;
            }
            Some(start..len)
        }

        // "A-B" => HTTP と同じく end 含むので Range には end+1 で落とす
        (false, false) => {
            let start = lhs.parse::<usize>().ok()?;
            let end_inclusive = rhs.parse::<usize>().ok()?;

            if end_inclusive < start {
                return None;
            }
            if start >= len {
                return None;
            }

            let end_exclusive = end_inclusive.saturating_add(1).min(len);
            Some(start..end_exclusive)
        }
    }
}

pub fn string_on_memory_size_hint(s: &String) -> usize {
    s.capacity() + std::mem::size_of::<String>()
}

pub mod byte_size_serde {
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format_bytes(*value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ByteSizeVisitor;

        impl<'de> Visitor<'de> for ByteSizeVisitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a non-negative integer byte size or a string like 100MiB")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(value)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    Err(E::custom("byte size must be non-negative"))
                } else {
                    Ok(value as u64)
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_size(value).map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(ByteSizeVisitor)
    }

    fn parse_size(input: &str) -> Result<u64, String> {
        let compact: String = input.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.is_empty() {
            return Err("byte size cannot be empty".to_string());
        }

        let split = compact
            .find(|c: char| !c.is_ascii_digit() && c != '_')
            .unwrap_or(compact.len());

        let number = compact[..split].replace('_', "");
        if number.is_empty() {
            return Err(format!("invalid byte size: {}", input));
        }
        let value = number
            .parse::<u64>()
            .map_err(|_| format!("invalid byte number: {}", input))?;

        let unit = compact[split..].to_ascii_lowercase();
        let multiplier = match unit.as_str() {
            "" | "b" | "byte" | "bytes" => 1_u64,
            "k" | "kb" => 1_000_u64,
            "m" | "mb" => 1_000_000_u64,
            "g" | "gb" => 1_000_000_000_u64,
            "t" | "tb" => 1_000_000_000_000_u64,
            "ki" | "kib" => 1_u64 << 10,
            "mi" | "mib" => 1_u64 << 20,
            "gi" | "gib" => 1_u64 << 30,
            "ti" | "tib" => 1_u64 << 40,
            _ => return Err(format!("unsupported byte unit: {}", unit)),
        };

        value
            .checked_mul(multiplier)
            .ok_or_else(|| "byte size is too large".to_string())
    }

    fn format_bytes(bytes: u64) -> String {
        const UNITS: [(&str, u64); 4] = [
            ("TiB", 1_u64 << 40),
            ("GiB", 1_u64 << 30),
            ("MiB", 1_u64 << 20),
            ("KiB", 1_u64 << 10),
        ];

        for (unit, size) in UNITS {
            if bytes >= size && bytes % size == 0 {
                return format!("{}{}", bytes / size, unit);
            }
        }

        format!("{}B", bytes)
    }
}