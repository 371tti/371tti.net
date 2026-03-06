use pulldown_latex::{Parser, RenderConfig, Storage, push_mathml};

use crate::render::slug::escape_html;

pub fn render_mathml(latex: &str, display: bool) -> String {
    let mut mathml = String::new();
    let storage = Storage::new();
    let rendered = push_mathml(
        &mut mathml,
        Parser::new(latex, &storage),
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
        let escaped = escape_html(latex);
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
