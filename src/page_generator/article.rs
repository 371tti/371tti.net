use dashmap::DashMap;
use gray_matter::engine::YAML;
use kurosabi::kurosabi::Context;
use latex2mathml::DisplayStyle;
use pulldown_cmark::{CodeBlockKind, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Deserialize;

use crate::{context::SiteContext, page_generator::{PageGenerator, err::ErrPage}, utils::html_escape};

pub struct ArticlePage {
    pub cache: DashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleMeta {
    pub title: String,
}

impl ArticlePage {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    pub async fn page(mut c: Context<SiteContext>) -> Context<SiteContext> {
        let base = c.req.path.get_field("*").unwrap_or("".into());
        let safe_base = base.replace("../", ""); // ディレクトリトラバーサル対策
        println!("Article requested: {}", safe_base);
        let path = if safe_base.ends_with("/") || safe_base.is_empty() {
            format!("data/blog/{}index.md", safe_base)
        } else {
            format!("data/blog/{}.md", safe_base)
        };
        
        let md = tokio::fs::read_to_string(&path).await;
        match md {
            Ok(md) => {
                let result = gray_matter::Matter::<YAML>::new().parse::<ArticleMeta>(&md);
                match result {
                    Ok(parsed) => {
                        let title = parsed.data.as_ref().map_or("Untitled", |data| data.title.as_str());
                        let mut md_to_html = MdToHTML::new(title);
                        let md = parsed.content;
                        md_to_html.convert(&md);
                        let html = md_to_html.generate();
                        let full_html = ArticlePage::addition(&html, title);
                        PageGenerator::base(c, &full_html, &safe_base, None)
                    }
                    Err(_e) => {
                        ErrPage::status_page(c, 500, "Failed to parse front matter")
                    }
                }
            }
            Err(_e) => {
                ErrPage::status_page(c, 404, "Article Not Found")
            }
        }
    }

    pub fn addition(html: &str, title: &str) -> String {
        format!(
"
<body>
<div class=\"full center scroll\">
<div class=\"card\" id=\"content\">
    <div class=\"box\">
        <h1>{}</h1>
        <hr>
    </div>

    <div class=\"article\">

        {}
    </div>

    <hr>
    <p class=\"center\">© 2024~ 371tti</p>
    <p class=\"center\">
        <a href=\"/\">top_page</a>　-　
        <a href=\"/terms\">terms</a>
    </p>

</div>
</div>
</body>
<script src=\"/rw-code.js\"></script>
",
title, html
        )
    }
}

struct MdToHTML {
    state: ConvertState,
    html: Vec<Section>,
    stop_push: bool,
}

pub struct Section {
    pub deepth: usize,
    pub title: String,
    pub content: String,
}

#[derive(PartialEq)]
enum ConvertState {
    Normal,
    InSectionTitleCollect,
    InSection,
}

impl MdToHTML {
    pub fn new(title: &str) -> Self {
        Self {
            state: ConvertState::Normal,
            html: vec![Section {
                deepth: 0,
                title: title.to_string(),
                content: String::new(),
            }],
            stop_push: false,
        }
    }

    fn push(&mut self, s: &str) {
        if self.stop_push {
            return;
        }
        if let Some(last) = self.html.last_mut() {
            if self.state == ConvertState::InSectionTitleCollect {
                last.title.push_str(s);
            }
            last.content.push_str(s);
        }
    }

    fn new_section(&mut self, deepth: usize) {
        self.state = ConvertState::InSection;
        self.html.push(
            Section { deepth, title: String::new(), content: String::new() }
        );
    }

    fn convert(&mut self, md: &str) {
        let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_WIKILINKS;

        let parser = Parser::new_ext(md, opts);

        for event in parser {
            match event {
                pulldown_cmark::Event::Start(tag) => self.tag_to_element(tag),
                pulldown_cmark::Event::End(tag_end) => self.end_to_element(tag_end),
                pulldown_cmark::Event::Text(cow_str) => self.push(&html_escape(&cow_str)),
                pulldown_cmark::Event::Code(cow_str) => self.push(&format!("<code>{}</code>", html_escape(&cow_str))),
                pulldown_cmark::Event::InlineMath(cow_str) => self.push(&format!("<span class=\"math-inline\">{}</span>", Self::latex_to_mathml(&cow_str, false))),
                pulldown_cmark::Event::DisplayMath(cow_str) => self.push(&format!("<div class=\"math-display\">{}</div>", Self::latex_to_mathml(&cow_str, true))),
                pulldown_cmark::Event::Html(_cow_str) => {}, // HTML は無視
                pulldown_cmark::Event::InlineHtml(cow_str) => self.push(&cow_str),
                pulldown_cmark::Event::FootnoteReference(cow_str) => self.push(&format!("<sup class=\"footnote-ref\">{}</sup>", cow_str)),
                pulldown_cmark::Event::SoftBreak => self.push("\n"),
                pulldown_cmark::Event::HardBreak => self.push("<br/>"),
                pulldown_cmark::Event::Rule => self.push("<hr/>"),
                pulldown_cmark::Event::TaskListMarker(ok) => {
                    if ok {
                        self.push("<input type=\"checkbox\" disabled checked/>");
                    } else {
                        self.push("<input type=\"checkbox\" disabled/>");
                    }
                }
            };
        }
    }

    fn generate(self) -> String {
        let len: usize= self.html.iter().map(|s| s.content.len()).sum();
        let addition_len: usize = self.html.len() * 50; // セクションタグ分の余裕
        let mut out = String::with_capacity(len + addition_len);
        for section in self.html {
            if section.content.is_empty() {
                continue;
            }
            out.push_str(&format!(
                "<section id=\"{}_{}\">\n{}\n</section>\n",
                section.deepth,
                section.title.replace(" ", "-"),
                section.content,
            ))
        }
        out
    }

    fn tag_to_element(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.push("<div class=\"indent\"><p>"),
            Tag::Heading { level, .. } => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                self.new_section(lvl);
                self.push(&format!("<h{}>", lvl));
                self.state = ConvertState::InSectionTitleCollect;
            }
            Tag::BlockQuote(kind) => {
                if let Some(kind) = kind {
                    match kind {
                        pulldown_cmark::BlockQuoteKind::Note => self.push("<blockquote class=\"quote-note\">"),
                        pulldown_cmark::BlockQuoteKind::Tip => self.push("<blockquote class=\"quote-tip\">"),
                        pulldown_cmark::BlockQuoteKind::Important => self.push("<blockquote class=\"quote-important\">"),
                        pulldown_cmark::BlockQuoteKind::Warning => self.push("<blockquote class=\"quote-warning\">"),
                        pulldown_cmark::BlockQuoteKind::Caution => self.push("<blockquote class=\"quote-caution\">"),
                    }
                } else {
                    self.push("<blockquote>");
                }
            }
            Tag::CodeBlock(lang) => {
                if let CodeBlockKind::Fenced(lang) = lang {
                    self.push(&format!("<pre><code class=\"language-{}\">", lang));
                } else {
                    self.push("<pre><code>");
                }
            }
            Tag::List(n) => {
                if let Some(n) = n {
                    self.push(&format!("<ol start=\"{}\">", n));
                } else {
                    self.push("<ul>");
                }
            }
            Tag::Item => self.push("<li>"),
            Tag::FootnoteDefinition(_) => self.push("<div>"),
            Tag::Table(_alignment) => {      // ここはとりあえず無視
                self.push("<table>");
            }
            Tag::TableHead => self.push("<thead>"),
            Tag::TableRow => self.push("<tr>"),
            Tag::TableCell => self.push("<td>"),
            Tag::HtmlBlock => {}, // HTML ブロックはそのまま出力されるのでここでは何もしない
            Tag::DefinitionList => self.push("<dl>"),
            Tag::DefinitionListTitle => self.push("<dt>"),
            Tag::DefinitionListDefinition => self.push("<dd>"),
            Tag::Emphasis => self.push("<em>"),
            Tag::Strong => self.push("<strong>"),
            Tag::Strikethrough => self.push("<del>"),
            Tag::Superscript => self.push("<sup>"),
            Tag::Subscript => self.push("<sub>"),
            Tag::Link { link_type, dest_url, title, id } => {
                let id_str = if !id.is_empty() {
                    format!(" id=\"{}\"", id)
                } else {
                    "".to_string()
                };
                let title_str = if !title.is_empty() {
                    format!(" title=\"{}\"", html_escape(&title))
                } else {
                    "".to_string()
                };
                match link_type {
                    pulldown_cmark::LinkType::Email => {
                        self.push(&format!("<a href=\"mailto:{}\"{}{}>", html_escape(&dest_url), title_str, id_str));
                    }
                    _ => {
                        self.push(&format!("<a href=\"{}\"{}{}>", html_escape(&dest_url), title_str, id_str));
                    }
                }
            }
            Tag::Image { link_type: _, dest_url, title, id } => {
                let id_str = if !id.is_empty() {
                    format!(" id=\"{}\"", id)
                } else {
                    "".to_string()
                };
                let alt_str = if !title.is_empty() {
                    format!(" alt=\"{}\"", html_escape(&title))
                } else {
                    "".to_string()
                };
                self.push(&format!("<div class=\"img\"><img src=\"{}\"{}{}>", html_escape(&dest_url), alt_str, id_str));
                self.stop_push = true;
            },
            _ => {},
        }
    }

    fn end_to_element(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.push("</p></div>"),
            TagEnd::Heading(heading_level) => {
                let lvl = match heading_level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                self.state = ConvertState::InSection;
                self.push(&format!("</h{}>", lvl));
            },
            TagEnd::BlockQuote(_block_quote_kind) => self.push("</blockquote>"),
            TagEnd::CodeBlock => self.push("</code></pre>"),
            TagEnd::HtmlBlock => {}, // HTML ブロックはそのまま出力されるのでここでは何もしない
            TagEnd::List(n) => {
                if n {
                    self.push("</ol>");
                } else {
                    self.push("</ul>");
                }
            }
            TagEnd::Item => self.push("</li>"),
            TagEnd::FootnoteDefinition => self.push("</div>"),
            TagEnd::DefinitionList => self.push("</dl>"),
            TagEnd::DefinitionListTitle => self.push("</dt>"),
            TagEnd::DefinitionListDefinition => self.push("</dd>"),
            TagEnd::Table => self.push("</table>"),
            TagEnd::TableHead => self.push("</thead>"),
            TagEnd::TableRow => self.push("</tr>"),
            TagEnd::TableCell => self.push("</td>"),
            TagEnd::Emphasis => self.push("</em>"),
            TagEnd::Strong => self.push("</strong>"),
            TagEnd::Strikethrough => self.push("</del>"),
            TagEnd::Superscript => self.push("</sup>"),
            TagEnd::Subscript => self.push("</sub>"),
            TagEnd::Link => self.push("</a>"),
            TagEnd::Image => {
                self.stop_push = false;
                self.push("</div>");
            },
            _ => {},
        }
    }

    fn latex_to_mathml(latex: &str, is_block: bool) -> String {
        let style = if is_block {
            DisplayStyle::Block
        } else {
            DisplayStyle::Inline
        };
        match latex2mathml::latex_to_mathml(latex, style) {
            Ok(mathml) => mathml,
            Err(_) => format!("<span class=\"math-error\">Invalid LaTeX: {}</span>", html_escape(latex)),
        }
    }
}