use pulldown_latex::{Parser, RenderConfig, Storage, push_mathml};
use tylax::{typst_to_latex_with_options, T2LOptions};

use crate::render::slug::escape_html;

pub fn render_mathml(latex: &str, display: bool) -> String {
    let mut mathml = String::new();
    let storage = Storage::new();
    let latex = add_typst_math(latex);

    let rendered = push_mathml(
        &mut mathml,
        Parser::new(&latex, &storage),
        RenderConfig::default(),
    )
    .is_ok();

    if rendered {
        if display {
            format!("<div class=\"math math-display\">{}</div>", mathml)
        } else {
            format!("<span class=\"math math-inline\">{}</span>", mathml)
        }
    } else {
        let escaped = escape_html(&latex);
        if display {
            format!(
                "<div class=\"math math-display\"><code>{}</code></div>",
                escaped
            )
        } else {
            format!(
                "<span class=\"math math-inline\"><code>{}</code></span>",
                escaped
            )
        }
    }
}

/// add \typst{...} support to math rendering
pub fn add_typst_math(latex: &str) -> String {
    const CMD: &str = r"\typst";

    if !latex.contains(CMD) {
        return latex.to_owned();
    }

    let mut out = String::with_capacity(latex.len());
    let mut search_from = 0;
    let mut last_emit = 0;

    while let Some(rel_pos) = latex[search_from..].find(CMD) {
        let cmd_start = search_from + rel_pos;
        let cmd_end = cmd_start + CMD.len();

        // \typstfoo を \typst と誤認しない
        if latex[cmd_end..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        {
            search_from = cmd_end;
            continue;
        }

        let arg_open = skip_ws(latex, cmd_end);

        if !latex[arg_open..].starts_with('{') {
            search_from = cmd_end;
            continue;
        }

        let Some(arg_close) = find_matching_brace(latex, arg_open) else {
            // 壊れた \typst{... はそのまま残す
            break;
        };

        out.push_str(&latex[last_emit..cmd_start]);

        let typst_src = &latex[arg_open + 1..arg_close];
        let converted = typst_math_to_latex(typst_src);

        out.push_str(&converted);

        search_from = arg_close + 1;
        last_emit = search_from;
    }

    out.push_str(&latex[last_emit..]);
    out
}

fn typst_math_to_latex(src: &str) -> String {
    let src = src.trim();

    if src.is_empty() {
        return String::new();
    }

    let options = T2LOptions {
        math_only: true,
        ..Default::default()
    };

    let converted = typst_to_latex_with_options(src, &options);
    let converted = converted.trim();

    // tylax は Result を返さないので、空になった場合だけ元を残す
    if converted.is_empty() {
        src.to_owned()
    } else {
        converted.to_owned()
    }
}

fn skip_ws(input: &str, mut pos: usize) -> usize {
    while let Some(ch) = input[pos..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }

        pos += ch.len_utf8();
    }

    pos
}

fn find_matching_brace(input: &str, open: usize) -> Option<usize> {
    debug_assert!(input[open..].starts_with('{'));

    let mut depth = 0usize;
    let mut escaped = false;

    for (rel_pos, ch) in input[open..].char_indices() {
        let pos = open + rel_pos;

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;

                if depth == 0 {
                    return Some(pos);
                }
            }
            _ => {}
        }
    }

    None
}