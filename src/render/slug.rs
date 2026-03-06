use markdown::mdast::Node;

pub fn slugify_id_unicode(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

pub fn escape_html(s: &str) -> String {
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

pub fn escape_attr(s: &str) -> String {
    escape_html(s)
}

pub fn extract_text(node: &Node) -> String {
    match node {
        Node::Text(text) => text.value.clone(),
        Node::InlineCode(code) => code.value.clone(),
        Node::InlineMath(math) => math.value.clone(),
        Node::Math(math) => math.value.clone(),
        Node::Break(_) => " ".to_string(),
        Node::Html(_) => String::new(),
        _ => node
            .children()
            .map(|children| {
                children
                    .iter()
                    .map(extract_text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default(),
    }
}
