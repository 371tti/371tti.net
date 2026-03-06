use markdown::mdast::{Blockquote, Node, Paragraph};

pub struct AlertBlock {
    pub class: &'static str,
    pub children: Vec<Node>,
}

pub fn transform_alert(blockquote: &Blockquote) -> Option<AlertBlock> {
    let Some(Node::Paragraph(first_para)) = blockquote.children.first() else {
        return None;
    };

    let (class, paragraph) = strip_marker_from_first_paragraph(first_para)?;
    let mut children = blockquote.children.clone();

    if paragraph.children.is_empty() {
        children.remove(0);
    } else {
        children[0] = Node::Paragraph(paragraph);
    }

    Some(AlertBlock { class, children })
}

fn alert_kind_class(kind: &str) -> Option<&'static str> {
    match kind {
        "note" => Some("quote-note"),
        "tip" => Some("quote-tip"),
        "warning" => Some("quote-warning"),
        "important" => Some("quote-important"),
        "caution" => Some("quote-caution"),
        _ => None,
    }
}

fn parse_marker_prefix(s: &str) -> Option<(&'static str, usize)> {
    let s = s.trim_start();
    if !s.starts_with("[!") {
        return None;
    }

    let close = s.find(']')?;
    let kind = s[2..close].trim().to_ascii_lowercase();
    let cls = alert_kind_class(&kind)?;
    Some((cls, close + 1))
}

fn strip_marker_from_first_paragraph(paragraph: &Paragraph) -> Option<(&'static str, Paragraph)> {
    let Some(Node::Text(first_text)) = paragraph.children.first() else {
        return None;
    };

    let (class, cut) = parse_marker_prefix(&first_text.value)?;
    let mut paragraph = paragraph.clone();

    if let Some(Node::Text(text)) = paragraph.children.first_mut() {
        let trimmed = text.value.trim_start();
        let rest = trimmed.chars().skip(cut).collect::<String>();
        text.value = rest.trim_start().to_string();
    }

    loop {
        let remove = match paragraph.children.first() {
            Some(Node::Text(text)) if text.value.is_empty() => true,
            Some(Node::Break(_)) => true,
            _ => false,
        };
        if remove {
            paragraph.children.remove(0);
        } else {
            break;
        }
    }

    if let Some(Node::Text(text)) = paragraph.children.first_mut() {
        text.value = text.value.trim_start().to_string();
    }

    Some((class, paragraph))
}
