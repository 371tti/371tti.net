use std::collections::HashMap;

use markdown::{
    Constructs, ParseOptions,
    mdast::{AlignKind, Definition, FootnoteDefinition, Node},
    to_mdast,
};

use crate::render::{
    alert::transform_alert,
    highlight::render_code_block,
    math::render_mathml,
    slug::{escape_attr, escape_html, extract_text, slugify_id_unicode},
};

pub fn render_markdown(md: &str) -> String {
    let parse_options = ParseOptions {
        constructs: Constructs {
            math_flow: true,
            math_text: true,
            ..Constructs::gfm()
        },
        ..ParseOptions::gfm()
    };

    match to_mdast(md, &parse_options) {
        Ok(ast) => {
            let mut state = RenderState::from_root(&ast);
            let mut out = String::new();
            render_root(&ast, &mut state, &mut out);
            out
        }
        Err(err) => format!(
            "<pre class=\"markdown-error\">{}</pre>",
            escape_html(&err.to_string())
        ),
    }
}

#[derive(Default)]
struct RenderState {
    used_slugs: HashMap<String, u32>,
    section_stack: Vec<u8>,
    definitions: HashMap<String, Definition>,
    footnotes: HashMap<String, FootnoteDefinition>,
    footnote_order: Vec<String>,
    footnote_numbers: HashMap<String, usize>,
    footnote_ref_counts: HashMap<String, usize>,
}

impl RenderState {
    fn from_root(root: &Node) -> Self {
        let mut state = Self::default();
        state.collect(root);
        state
    }

    fn collect(&mut self, node: &Node) {
        match node {
            Node::Definition(def) => {
                self.definitions.insert(def.identifier.clone(), def.clone());
            }
            Node::FootnoteDefinition(def) => {
                self.footnotes.insert(def.identifier.clone(), def.clone());
            }
            _ => {}
        }

        if let Some(children) = node.children() {
            for child in children {
                self.collect(child);
            }
        }
    }

    fn next_slug(&mut self, heading: &Node) -> String {
        let base = slugify_id_unicode(&extract_text(heading));
        match self.used_slugs.get_mut(&base) {
            None => {
                self.used_slugs.insert(base.clone(), 1);
                base
            }
            Some(count) => {
                *count += 1;
                format!("{}-{}", base, *count)
            }
        }
    }

    fn close_sections_at_or_above(&mut self, depth: u8, out: &mut String) {
        while let Some(top) = self.section_stack.last().copied() {
            if top >= depth {
                out.push_str("</section>\n");
                self.section_stack.pop();
            } else {
                break;
            }
        }
    }

    fn close_all_sections(&mut self, out: &mut String) {
        while self.section_stack.pop().is_some() {
            out.push_str("</section>\n");
        }
    }

    fn register_footnote_reference(&mut self, identifier: &str) -> (usize, usize) {
        let number = match self.footnote_numbers.get(identifier) {
            Some(number) => *number,
            None => {
                let number = self.footnote_order.len() + 1;
                self.footnote_order.push(identifier.to_string());
                self.footnote_numbers.insert(identifier.to_string(), number);
                number
            }
        };

        let ref_count = self
            .footnote_ref_counts
            .entry(identifier.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);

        (number, *ref_count)
    }
}

fn render_root(node: &Node, state: &mut RenderState, out: &mut String) {
    match node {
        Node::Root(root) => {
            render_flow_nodes(&root.children, state, out);
            state.close_all_sections(out);
            render_footnotes(state, out);
        }
        _ => render_flow_node(node, state, out),
    }
}

fn render_flow_nodes(nodes: &[Node], state: &mut RenderState, out: &mut String) {
    for node in nodes {
        render_flow_node(node, state, out);
    }
}

fn render_flow_node(node: &Node, state: &mut RenderState, out: &mut String) {
    match node {
        Node::Heading(heading) => {
            state.close_sections_at_or_above(heading.depth, out);
            let id = state.next_slug(node);
            let id_esc = escape_attr(&id);
            out.push_str(&format!(
                "<section class=\"md-section\" id=\"{}\">\n",
                id_esc
            ));
            state.section_stack.push(heading.depth);
            out.push_str(&format!("<h{}>", heading.depth));
            out.push_str(&format!("<a class=\"heading-link\" href=\"#{}\">", id_esc));
            render_inline_nodes(&heading.children, state, out);
            out.push_str("</a>");
            out.push_str(&format!("</h{}>\n", heading.depth));
        }
        Node::Paragraph(paragraph) => {
            out.push_str("<p>");
            render_inline_nodes(&paragraph.children, state, out);
            out.push_str("</p>\n");
        }
        Node::Blockquote(blockquote) => {
            if let Some(alert) = transform_alert(blockquote) {
                out.push_str(&format!("<blockquote class=\"{}\">\n", alert.class));
                render_flow_nodes(&alert.children, state, out);
                out.push_str("</blockquote>\n");
            } else {
                out.push_str("<blockquote>\n");
                render_flow_nodes(&blockquote.children, state, out);
                out.push_str("</blockquote>\n");
            }
        }
        Node::Code(code) => {
            out.push_str(&render_code_block(&code.value, code.lang.as_deref()));
        }
        Node::Math(math) => {
            out.push_str(&render_mathml(&math.value, true));
            out.push('\n');
        }
        Node::Html(html) => {
            out.push_str(&html.value);
            out.push('\n');
        }
        Node::List(list) => render_list(list, state, out),
        Node::Table(table) => render_table(table, state, out),
        Node::ThematicBreak(_) => out.push_str("<hr />\n"),
        Node::Definition(_) | Node::FootnoteDefinition(_) => {}
        Node::Text(text) => {
            out.push_str(&escape_html(&text.value));
            out.push('\n');
        }
        Node::Yaml(_) | Node::Toml(_) => {}
        _ => {
            if let Some(children) = node.children() {
                render_flow_nodes(children, state, out);
            }
        }
    }
}

fn render_inline_nodes(nodes: &[Node], state: &mut RenderState, out: &mut String) {
    for node in nodes {
        render_inline_node(node, state, out);
    }
}

fn render_inline_node(node: &Node, state: &mut RenderState, out: &mut String) {
    match node {
        Node::Text(text) => out.push_str(&render_text_markup(&text.value)),
        Node::InlineCode(code) => {
            out.push_str("<code>");
            out.push_str(&escape_html(&code.value));
            out.push_str("</code>");
        }
        Node::InlineMath(math) => out.push_str(&render_mathml(&math.value, false)),
        Node::Break(_) => out.push_str("<br />\n"),
        Node::Emphasis(emphasis) => {
            out.push_str("<em>");
            render_inline_nodes(&emphasis.children, state, out);
            out.push_str("</em>");
        }
        Node::Strong(strong) => {
            out.push_str("<strong>");
            render_inline_nodes(&strong.children, state, out);
            out.push_str("</strong>");
        }
        Node::Delete(delete) => {
            out.push_str("<del>");
            render_inline_nodes(&delete.children, state, out);
            out.push_str("</del>");
        }
        Node::Link(link) => {
            out.push_str("<a href=\"");
            out.push_str(&escape_attr(&link.url));
            out.push('"');
            if let Some(title) = &link.title {
                out.push_str(" title=\"");
                out.push_str(&escape_attr(title));
                out.push('"');
            }
            out.push('>');
            render_inline_nodes(&link.children, state, out);
            out.push_str("</a>");
        }
        Node::Image(image) => {
            out.push_str("<img src=\"");
            out.push_str(&escape_attr(&image.url));
            out.push_str("\" alt=\"");
            out.push_str(&escape_attr(&image.alt));
            out.push('"');
            if let Some(title) = &image.title {
                out.push_str(" title=\"");
                out.push_str(&escape_attr(title));
                out.push('"');
            }
            out.push_str(" />");
        }
        Node::LinkReference(link) => {
            if let Some(def) = state.definitions.get(&link.identifier) {
                out.push_str("<a href=\"");
                out.push_str(&escape_attr(&def.url));
                out.push('"');
                if let Some(title) = &def.title {
                    out.push_str(" title=\"");
                    out.push_str(&escape_attr(title));
                    out.push('"');
                }
                out.push('>');
                render_inline_nodes(&link.children, state, out);
                out.push_str("</a>");
            } else {
                render_inline_nodes(&link.children, state, out);
            }
        }
        Node::ImageReference(image) => {
            if let Some(def) = state.definitions.get(&image.identifier) {
                out.push_str("<img src=\"");
                out.push_str(&escape_attr(&def.url));
                out.push_str("\" alt=\"");
                out.push_str(&escape_attr(&image.alt));
                out.push('"');
                if let Some(title) = &def.title {
                    out.push_str(" title=\"");
                    out.push_str(&escape_attr(title));
                    out.push('"');
                }
                out.push_str(" />");
            } else {
                out.push_str(&escape_html(&image.alt));
            }
        }
        Node::FootnoteReference(footnote) => {
            let (number, ref_index) = state.register_footnote_reference(&footnote.identifier);
            let slug = slugify_id_unicode(&footnote.identifier);
            out.push_str(&format!(
                "<sup><a href=\"#fn-{}\" id=\"fnref-{}-{}\" data-footnote-ref=\"\">{}</a></sup>",
                escape_attr(&slug),
                escape_attr(&slug),
                ref_index,
                number
            ));
        }
        Node::Html(html) => out.push_str(&html.value),
        Node::Code(code) => out.push_str(&render_code_block(&code.value, code.lang.as_deref())),
        _ => {
            if let Some(children) = node.children() {
                render_inline_nodes(children, state, out);
            }
        }
    }
}

fn render_list(list: &markdown::mdast::List, state: &mut RenderState, out: &mut String) {
    let is_task_list = list
        .children
        .iter()
        .any(|child| matches!(child, Node::ListItem(item) if item.checked.is_some()));

    if list.ordered {
        out.push_str("<ol");
        if is_task_list {
            out.push_str(" class=\"task-list\"");
        }
        if let Some(start) = list.start.filter(|start| *start != 1) {
            out.push_str(&format!(" start=\"{}\"", start));
        }
        out.push_str(">\n");
    } else {
        if is_task_list {
            out.push_str("<ul class=\"task-list\">\n");
        } else {
            out.push_str("<ul>\n");
        }
    }

    for child in &list.children {
        if let Node::ListItem(item) = child {
            render_list_item(item, state, out);
        }
    }

    if list.ordered {
        out.push_str("</ol>\n");
    } else {
        out.push_str("</ul>\n");
    }
}

fn render_list_item(item: &markdown::mdast::ListItem, state: &mut RenderState, out: &mut String) {
    if item.checked.is_some() {
        out.push_str("<li class=\"task-list-item\">");
    } else {
        out.push_str("<li>");
    }
    if let Some(checked) = item.checked {
        out.push_str("<input class=\"task-list-checkbox\" type=\"checkbox\"");
        if checked {
            out.push_str(" checked=\"\"");
        }
        out.push_str(" disabled=\"\" /> ");
    }

    let is_tight_paragraph = !item.spread
        && item.children.len() == 1
        && matches!(item.children.first(), Some(Node::Paragraph(_)));

    if is_tight_paragraph {
        if let Some(Node::Paragraph(paragraph)) = item.children.first() {
            render_inline_nodes(&paragraph.children, state, out);
        }
        out.push_str("</li>\n");
        return;
    }

    out.push('\n');
    render_flow_nodes(&item.children, state, out);
    out.push_str("</li>\n");
}

fn render_table(table: &markdown::mdast::Table, state: &mut RenderState, out: &mut String) {
    out.push_str("<table>\n");
    let mut rows = table.children.iter();

    if let Some(Node::TableRow(header)) = rows.next() {
        out.push_str("<thead>\n<tr>");
        for (index, cell) in header.children.iter().enumerate() {
            render_table_cell(cell, table.align.get(index), true, state, out);
        }
        out.push_str("</tr>\n</thead>\n");
    }

    let body_rows = rows.collect::<Vec<_>>();
    if !body_rows.is_empty() {
        out.push_str("<tbody>\n");
        for row in body_rows {
            if let Node::TableRow(row) = row {
                out.push_str("<tr>");
                for (index, cell) in row.children.iter().enumerate() {
                    render_table_cell(cell, table.align.get(index), false, state, out);
                }
                out.push_str("</tr>\n");
            }
        }
        out.push_str("</tbody>\n");
    }

    out.push_str("</table>\n");
}

fn render_table_cell(
    node: &Node,
    align: Option<&AlignKind>,
    header: bool,
    state: &mut RenderState,
    out: &mut String,
) {
    let tag = if header { "th" } else { "td" };
    out.push('<');
    out.push_str(tag);
    if let Some(style) = align_to_style(align) {
        out.push_str(" style=\"");
        out.push_str(style);
        out.push('"');
    }
    out.push('>');
    if let Node::TableCell(cell) = node {
        render_inline_nodes(&cell.children, state, out);
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}

fn align_to_style(align: Option<&AlignKind>) -> Option<&'static str> {
    match align.copied().unwrap_or(AlignKind::None) {
        AlignKind::Left => Some("text-align:left"),
        AlignKind::Right => Some("text-align:right"),
        AlignKind::Center => Some("text-align:center"),
        AlignKind::None => None,
    }
}

fn render_footnotes(state: &mut RenderState, out: &mut String) {
    if state.footnote_order.is_empty() {
        return;
    }

    out.push_str("<section data-footnotes=\"\" class=\"footnotes\">\n");
    out.push_str("<h2 id=\"footnote-label\" class=\"sr-only\">Footnotes</h2>\n");
    out.push_str("<ol>\n");

    for identifier in state.footnote_order.clone() {
        let slug = slugify_id_unicode(&identifier);
        out.push_str(&format!("<li id=\"fn-{}\">\n", escape_attr(&slug)));
        if let Some(definition) = state.footnotes.get(&identifier).cloned() {
            render_flow_nodes(&definition.children, state, out);
        }

        let backrefs = state
            .footnote_ref_counts
            .get(&identifier)
            .copied()
            .unwrap_or(0);
        if backrefs > 0 {
            out.push_str("<p>");
            for index in 1..=backrefs {
                if index > 1 {
                    out.push(' ');
                }
                out.push_str(&format!(
                    "<a href=\"#fnref-{}-{}\" data-footnote-backref=\"\" aria-label=\"Back to content\" class=\"data-footnote-backref\">↩</a>",
                    escape_attr(&slug),
                    index
                ));
            }
            out.push_str("</p>\n");
        }
        out.push_str("</li>\n");
    }

    out.push_str("</ol>\n</section>\n");
}

fn render_text_markup(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if (ch == '=' || ch == '|') && chars.peek() == Some(&ch) {
            chars.next();
            let marker = ch;
            let mut inner = String::new();
            let mut found_end = false;

            while let Some(next) = chars.next() {
                if next == marker && chars.peek() == Some(&marker) {
                    chars.next();
                    found_end = true;
                    break;
                }
                inner.push(next);
            }

            if found_end && !inner.is_empty() {
                let inner_html = render_text_markup(&inner);
                if marker == '=' {
                    out.push_str("<mark>");
                    out.push_str(&inner_html);
                    out.push_str("</mark>");
                } else {
                    out.push_str("<span class=\"spoiler\">");
                    out.push_str(&inner_html);
                    out.push_str("</span>");
                }
            } else {
                out.push(marker);
                out.push(marker);
                out.push_str(&escape_html(&inner));
            }
            continue;
        }

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
