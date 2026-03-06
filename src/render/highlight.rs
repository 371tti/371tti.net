use std::sync::LazyLock;

use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{IncludeBackground, styled_line_to_highlighted_html},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use crate::render::slug::{escape_attr, escape_html};

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

pub fn render_code_block(code: &str, lang: Option<&str>) -> String {
    let language_class = lang
        .filter(|value| !value.is_empty())
        .map(|value| format!(" class=\"language-{}\"", escape_attr(value)))
        .unwrap_or_default();

    let highlighted = highlight(code, lang).unwrap_or_else(|| escape_html(code));
    format!(
        "<pre class=\"code-block\"><code{}>{}</code></pre>\n",
        language_class, highlighted
    )
}

fn highlight(code: &str, lang: Option<&str>) -> Option<String> {
    let syntax_set = &*SYNTAX_SET;
    let theme_set = &*THEME_SET;
    let theme = theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())?;

    let syntax = lang
        .and_then(|value| syntax_set.find_syntax_by_token(value))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut out = String::new();

    for line in LinesWithEndings::from(code) {
        let newline = if line.ends_with("\r\n") {
            "\r\n"
        } else if line.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let content = line.strip_suffix(newline).unwrap_or(line);
        let regions = highlighter.highlight_line(content, syntax_set).ok()?;
        let html = styled_line_to_highlighted_html(&regions, IncludeBackground::No).ok()?;
        out.push_str(&html);
        out.push_str(newline);
    }

    Some(out)
}
