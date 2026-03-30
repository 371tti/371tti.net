use resvg::{render, tiny_skia, usvg};

use crate::{DEFAULT_FONT_DATA, DEFAULT_FONT_NAME};

#[derive(Debug, Clone)]
pub struct OgpData {
    path: String,
    title: String,
    description: String,
    authors: Vec<String>,
    tags: Vec<String>,
}

/// い
impl OgpData {
    pub fn new(
        path: String,
        title: String,
        description: String,
        authors: Vec<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            path,
            title,
            description,
            authors,
            tags,
        }
    }

    pub fn render_svg(&self) -> String {
        build_svg(self)
    }

    pub fn render_png(&self) -> Option<Vec<u8>> {
        let svg_data = self.render_svg();
        let mut opt = usvg::Options::default();

        opt.fontdb_mut().load_font_data(DEFAULT_FONT_DATA.to_vec());
        opt.font_family = DEFAULT_FONT_NAME.to_string();
        let tree = usvg::Tree::from_str(&svg_data, &opt).ok()?;
        let size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())?;

        render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap.encode_png().ok()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 半角=1, 全角=2 のざっくり幅
fn char_units(c: char) -> usize {
    if c.is_ascii() { 1 } else { 2 }
}

fn text_units(s: &str) -> usize {
    s.chars().map(char_units).sum()
}

/// 1行に収まらなければ末尾を … にする
fn truncate_to_units(s: &str, max_units: usize) -> String {
    truncate_to_units_with_used(s, max_units).0
}

/// 1行に収まらなければ末尾を … にする。使った幅も返す。
fn truncate_to_units_with_used(s: &str, max_units: usize) -> (String, usize) {
    if max_units == 0 {
        return (String::new(), 0);
    }

    let total = text_units(s);
    if total <= max_units {
        return (s.to_string(), total);
    }

    let ellipsis = '…';
    let ellipsis_units = 2;
    let mut out = String::new();
    let mut used = 0;

    for c in s.chars() {
        let cu = char_units(c);
        if used + cu + ellipsis_units > max_units {
            out.push(ellipsis);
            used += ellipsis_units;
            break;
        }
        out.push(c);
        used += cu;
    }

    (out, used)
}

/// 最大 max_lines 行まで自動改行し、最後の行は必要なら … で切る
fn wrap_lines(s: &str, line_units: usize, max_lines: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut lines = Vec::new();
    let mut i = 0;

    while i < chars.len() && lines.len() < max_lines {
        let is_last_line = lines.len() + 1 == max_lines;
        let mut used = 0;
        let mut buf = String::new();
        let mut last_break: Option<(usize, usize)> = None;
        let start_i = i;

        while i < chars.len() {
            let c = chars[i];
            let cu = char_units(c);

            let reserve = if is_last_line { 2 } else { 0 };
            if used + cu + reserve > line_units {
                break;
            }

            buf.push(c);
            used += cu;

            if c.is_whitespace() || "、。，．,.!！?？/／-‐ー".contains(c) {
                last_break = Some((i, buf.len()));
            }

            i += 1;
        }

        if i == chars.len() {
            lines.push(buf.trim().to_string());
            break;
        }

        if is_last_line {
            let mut trimmed = buf.trim_end().to_string();
            if !trimmed.ends_with('…') {
                trimmed.push('…');
            }
            lines.push(trimmed);
            break;
        }

        if i < chars.len() && i > start_i {
            if let Some((break_i, buf_len_at_break)) = last_break {
                if break_i >= start_i {
                    buf.truncate(buf_len_at_break);
                    i = break_i + 1;
                }
            }
        }

        let trimmed = buf.trim().to_string();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        } else {
            // 念のため
            let c = chars[i];
            i += 1;
            lines.push(c.to_string());
        }
    }

    while lines.len() < max_lines {
        lines.push(String::new());
    }

    lines
}

fn join_authors(authors: &[String]) -> String {
    authors.join(" · ")
}

/// 利用可能な幅に収まるように tags を詰める。
/// 収まらない場合は末尾に "+N tags" を追加して要約する。
fn fit_tags_to_width(tags: &[String], max_width: f32, gap: f32) -> Vec<String> {
    if tags.is_empty() || max_width <= 0.0 {
        return Vec::new();
    }

    let all_tags = tags.to_vec();
    if estimate_tags_total_width(&all_tags, gap) <= max_width {
        return all_tags;
    }

    let mut visible_count = tags.len();
    while visible_count > 0 {
        visible_count -= 1;
        let hidden_count = tags.len() - visible_count;

        let mut candidate = tags[..visible_count].to_vec();
        candidate.push(format!("+{} tags", hidden_count));

        if estimate_tags_total_width(&candidate, gap) <= max_width {
            return candidate;
        }
    }

    let summary_only = format!("+{} tags", tags.len());
    if estimate_tag_badge_width(&summary_only) <= max_width {
        return vec![summary_only];
    }

    Vec::new()
}

/// 等幅フォント前提のざっくりピクセル幅
fn estimate_px_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|c| {
            if c.is_ascii() {
                font_size * 0.62
            } else {
                font_size * 1.0
            }
        })
        .sum()
}

fn estimate_author_px_width(text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|c| {
            if c.is_ascii() {
                font_size * 0.50
            } else {
                font_size * 0.92
            }
        })
        .sum()
}

fn estimate_tag_badge_width(tag: &str) -> f32 {
    let font_size = 28.0;
    let padding_x = 22.0;
    estimate_px_width(tag, font_size) + padding_x * 2.0
}

fn estimate_tags_total_width(tags: &[String], gap: f32) -> f32 {
    let mut total = 0.0;

    for (idx, tag) in tags.iter().enumerate() {
        if idx > 0 {
            total += gap;
        }
        total += estimate_tag_badge_width(tag);
    }

    total
}

fn fit_authors_to_units_for_tags(
    joined_authors: &str,
    max_units: usize,
    author_font_size: f32,
    tags: &[String],
    content_width: f32,
    author_tag_gap: f32,
) -> String {
    if tags.is_empty() {
        return truncate_to_units(joined_authors, max_units);
    }

    let summary_label = format!("+{} tags", tags.len());
    let summary_width = estimate_tag_badge_width(&summary_label);
    let max_author_width = (content_width - author_tag_gap - summary_width).max(0.0);

    let mut units = max_units;
    loop {
        let candidate = truncate_to_units(joined_authors, units);
        if estimate_author_px_width(&candidate, author_font_size) <= max_author_width || units == 0
        {
            return candidate;
        }
        units -= 1;
    }
}

fn expand_authors_for_fixed_tags(
    joined_authors: &str,
    max_units: usize,
    author_font_size: f32,
    tags: &[String],
    fixed_display_tags: &[String],
    content_width: f32,
    author_tag_gap: f32,
    tag_gap: f32,
    fallback_authors: &str,
) -> String {
    let mut best = fallback_authors.to_string();

    for units in 0..=max_units {
        let candidate = truncate_to_units(joined_authors, units);
        let authors_px = estimate_author_px_width(&candidate, author_font_size);
        let available_for_tags = (content_width - (authors_px + author_tag_gap)).max(0.0);
        let candidate_tags = fit_tags_to_width(tags, available_for_tags, tag_gap);

        if candidate_tags == fixed_display_tags {
            best = candidate;
        }
    }

    best
}

fn render_tags(tags: &[String], start_x: f32, y: f32, height: f32, gap: f32) -> String {
    let mut x = start_x;
    let mut out = String::new();

    for tag in tags {
        let width = estimate_tag_badge_width(tag);
        let rx = height / 2.0;
        let escaped = escape_xml(tag);

        let text_x = x + width / 2.0;
        let text_y = y + 36.0;

        out.push_str(&format!(
            r##"
    <rect x="{x:.1}" y="{y:.1}" width="{width:.1}" height="{height:.1}" rx="{rx:.1}" fill="#282828"/>
    <text x="{text_x:.1}" y="{text_y:.1}"
          text-anchor="middle"
          font-size="28"
          font-weight="700"
          fill="#a9b7e6">{escaped}</text>
"##,
        ));

        x += width + gap;
    }

    out
}

fn build_svg(data: &OgpData) -> String {
    const CONTENT_LEFT: f32 = 80.0;
    const CONTENT_RIGHT: f32 = 1128.0;
    const AUTHOR_TAG_GAP: f32 = 32.0;
    const TAG_GAP: f32 = 14.0;
    const AUTHOR_MAX_UNITS: usize = 45;
    const AUTHOR_FONT_SIZE: f32 = 40.0;
    const TITLE_LINE1_Y: f32 = 236.0;
    const TITLE_LINE2_Y: f32 = 324.0;
    const TITLE_SINGLE_LINE_Y: f32 = (TITLE_LINE1_Y + TITLE_LINE2_Y) / 2.0;

    let path = truncate_to_units(&data.path, 64);
    let title_lines = wrap_lines(&data.title, 27, 2);
    let desc_lines = wrap_lines(&data.description, 64, 2);
    let title_line1_y = if title_lines[1].is_empty() {
        TITLE_SINGLE_LINE_Y
    } else {
        TITLE_LINE1_Y
    };

    let joined_authors = join_authors(&data.authors);
    let content_width = CONTENT_RIGHT - CONTENT_LEFT;
    let seed_authors_text = fit_authors_to_units_for_tags(
        &joined_authors,
        AUTHOR_MAX_UNITS,
        AUTHOR_FONT_SIZE,
        &data.tags,
        content_width,
        AUTHOR_TAG_GAP,
    );
    let seed_authors_px = estimate_author_px_width(&seed_authors_text, AUTHOR_FONT_SIZE);
    let seed_available_for_tags = (content_width - (seed_authors_px + AUTHOR_TAG_GAP)).max(0.0);
    let display_tags = fit_tags_to_width(&data.tags, seed_available_for_tags, TAG_GAP);

    // tags の表示数を維持しつつ、authors を可能な限り広げて余白を減らす。
    let authors_text = expand_authors_for_fixed_tags(
        &joined_authors,
        AUTHOR_MAX_UNITS,
        AUTHOR_FONT_SIZE,
        &data.tags,
        &display_tags,
        content_width,
        AUTHOR_TAG_GAP,
        TAG_GAP,
        &seed_authors_text,
    );

    let path = escape_xml(&path);
    let title_line1 = escape_xml(&title_lines[0]);
    let title_line2 = escape_xml(&title_lines[1]);
    let desc_line1 = escape_xml(&desc_lines[0]);
    let desc_line2 = escape_xml(&desc_lines[1]);
    let authors = escape_xml(&authors_text);

    // tags は authors 直後を優先。収まらない場合のみ右寄せに倒す。
    let authors_px = estimate_author_px_width(&authors_text, AUTHOR_FONT_SIZE);
    let tags_total_px = estimate_tags_total_width(&display_tags, TAG_GAP);
    let tags_start_x =
        (CONTENT_LEFT + authors_px + AUTHOR_TAG_GAP).min(CONTENT_RIGHT - tags_total_px);

    let tags_svg = render_tags(&display_tags, tags_start_x, 524.0, 52.0, TAG_GAP);

    format!(
        r##"
<svg width="1200" height="630" viewBox="0 0 1200 630" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg-grad" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#121212"/>
      <stop offset="100%" stop-color="#1a1a1a"/>
    </linearGradient>

    <linearGradient id="accent-line" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#6f7a95" stop-opacity="0"/>
      <stop offset="35%" stop-color="#6f7a95" stop-opacity="0.95"/>
      <stop offset="100%" stop-color="#a9b7e6" stop-opacity="0.95"/>
    </linearGradient>
  </defs>

  <rect width="1200" height="630" fill="url(#bg-grad)"/>

  <g stroke="#282828" stroke-opacity="0.6" stroke-width="1">
    <line x1="80" y1="0" x2="80" y2="630"/>
    <line x1="320" y1="0" x2="320" y2="630"/>
    <line x1="560" y1="0" x2="560" y2="630"/>
    <line x1="800" y1="0" x2="800" y2="630"/>
    <line x1="1040" y1="0" x2="1040" y2="630"/>

    <line x1="0" y1="120" x2="1200" y2="120"/>
    <line x1="0" y1="315" x2="1200" y2="315"/>
    <line x1="0" y1="510" x2="1200" y2="510"/>
  </g>

  <!-- path -->
  <text x="80" y="86"
            font-size="32"
        font-weight="600"
                fill="#7f8db1">{path}</text>

  <!-- accent -->
  <rect x="80" y="108" width="240" height="4" rx="2" fill="url(#accent-line)"/>

  <!-- title -->
    <text x="80" y="{title_line1_y:.1}"
        font-size="76"
        font-weight="800"
        fill="#cccccc">{title_line1}</text>

  <text x="80" y="324"
        font-size="76"
        font-weight="800"
        fill="#cccccc">{title_line2}</text>

  <!-- description -->
  <text x="80" y="408"
            font-size="32"
        font-weight="500"
                fill="#b0b0b0">{desc_line1}</text>

  <text x="80" y="446"
            font-size="32"
        font-weight="500"
                fill="#b0b0b0">{desc_line2}</text>

  <!-- bottom row -->
  <rect x="80" y="492" width="1048" height="2" fill="#282828"/>
  <rect x="80" y="492" width="220" height="2" fill="#6f7a95"/>

  <!-- authors -->
  <text x="80" y="566"
        font-size="40"
        font-weight="600"
        fill="#999999">{authors}</text>

  <!-- tags -->
  <g>
    {tags_svg}
  </g>
</svg>
"##,
    )
}
