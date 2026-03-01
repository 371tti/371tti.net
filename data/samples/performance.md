Server Side でのシンタックスハイライトの速度を測るためのサンプル
```rust

use std::{cell::RefCell, collections::HashMap};

use comrak::{Arena, Options, format_html_with_plugins, nodes::{AstNode, NodeValue}, options::Plugins, parse_document, plugins::syntect::SyntectAdapter};
use kurosabi::{connection::file::{DirEntryInfo, FileContentBuilder}, http::{HttpMethod, HttpStatusCode}, server::tokio::KurosabiTokioServerBuilder, utils::{url_decode_fast, url_encode}};
use pulldown_latex::{Parser, RenderConfig, Storage, push_mathml};
use tokio::io::AsyncReadExt;

pub const BASE_DIR: &str = "./data/";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    KurosabiTokioServerBuilder::default()
        .bind([0, 0, 0, 0])
        .port(8080)
        .router_and_build(|conn| async move {
            match conn.req.method() {
                HttpMethod::GET => {
                    match conn.path_segs().as_ref() {
                        ["raw", path @ ..] => {
                            let content = FileContentBuilder::base(BASE_DIR).path_url_segs(path).inline();
                            conn.file_body(content).await.unwrap_or_else(|p| p.connection)
                        }
                        ["style.css"] => conn.css_body(include_str!("../data/style.css").to_string()),
                        ["menu.js"] => conn.js_body(include_str!("../data/menu.js").to_string()),
                        ["optimizer.js"] => conn.js_body(include_str!("../data/optimizer.js").to_string()),
                        [path @ ..] => {
                            match docs_routing(path).await {
                                Ok(Some(html)) => conn.html_body(html),
                                Ok(None) => {
                                    let redirect_path = "/raw/".to_string() + &path.join("/");
                                    conn.redirect(redirect_path)
                                }
                                Err(_) => conn.set_status_code(HttpStatusCode::NotFound).no_body(),
                            }
                        }
                        _ => conn.set_status_code(HttpStatusCode::NotFound).no_body(),
                    }
                }
                _ => conn.set_status_code(HttpStatusCode::MethodNotAllowed).no_body()
            }
        })
        .run().await
}

/// Err => 404
/// Ok(None) => redirect raw endpoint
/// Ok(Some(bytes)) => serve bytes
pub async fn docs_routing(path: &[&str]) -> std::io::Result<Option<String>> {
    let builder = match FileContentBuilder::base(BASE_DIR).path_url_segs(path).check_file_exists().await {
        Ok(f) => f,
        Err(d) => match d {
            Some(dir) => return Ok(Some(md_dir_render(dir, path).await)),
            None => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")),
        }
    };
    let mut file = builder.build().await?;
    if file.mime_type != "text/markdown; charset=utf-8" {
        return Ok(None);
    }
    let mut buf = String::new();
    let _bytes = file.file.read_to_string(&mut buf).await?;
    let content = md_to_html_gfm_highlight(&buf);
    let html = format!(
        include_str!("../data/index.html"),
        title = path.last().unwrap_or(&"Undefined"),
        content = content
    );
    Ok(Some(html))
}

/// ディレクトリ一覧を Markdown で生成して HTML 化
/// index.md があれば末尾に追加
pub async fn md_dir_render(dir: Vec<DirEntryInfo>, path: &[&str]) -> String {
    let mut path_with_index = if path == [""] {
        vec![]
    } else {
        path.to_vec()
    };
    path_with_index.push("index.md");
    let index_md: Option<String> = match FileContentBuilder::base(BASE_DIR).path_url_segs(&path_with_index).build().await {
        Ok(mut file) => {
            if file.mime_type != "text/markdown; charset=utf-8" {
                None
            } else {
                let mut buf = String::new();
                match file.file.read_to_string(&mut buf).await {
                    Err(_) => None,
                    Ok(_) => Some(buf),
                }
            }
        }
        Err(_) => None,
    };
    let mut files: Vec<&str> = Vec::new();
    let mut dirs: Vec<&str> = Vec::new();
    for entry in dir.iter() {
        if entry.kind.is_dir() {
            let opt_dir_name = entry.path.file_name().and_then(|n| n.to_str());
            if let Some(dir_name) = opt_dir_name {
                if ! dir_name.starts_with(".") {
                    dirs.push(dir_name);
                }
            }
        } else if entry.kind.is_file() {
            let opt_file_name = entry.path.file_name().and_then(|n| n.to_str());
            if let Some(file_name) = opt_file_name {
                if ! file_name.starts_with(".") {
                    files.push(file_name);
                }
            }
        }
    }
    files.sort_unstable();
    dirs.sort_unstable();
    let mut md = format!("# Index of /{}",
        path.iter().enumerate().map(|(i, p)| {
            let link = if i == path.len() - 1 {
                // 最後はリンクなし
                p.to_string()
            } else {
                let href = format!("/{}", url_encode(&path[..=i].join("/")));
                format!("[{}]({})", p, href)
            };
            link
        }).collect::<Vec<_>>().join("/")
    );
    md.push_str(
        &match dirs.iter().map(|d| format!("- [{}]({}/{})", d, url_encode(path.last().map_or("", |v| v)), url_encode(d))).collect::<Vec<_>>().join("\n") {
            s if s.is_empty() => "".to_string(),
            s => format!("\n\n# Directories\n\n{}", s),
        }
    );
    md.push_str(
        &match files.iter().map(|f| format!("- [{}]({}/{})", f, url_encode(path.last().map_or("", |v| v)), url_encode(f))).collect::<Vec<_>>().join("\n") {
            s if s.is_empty() => "".to_string(),
            s => format!("\n\n# Files\n\n{}", s),
        }
    );
    md.push_str(
        &match index_md {
            Some(content) => format!("\n\n---\n\n{}", content),
            None => "".to_string(),
        }
    );
    let content = md_to_html_gfm_highlight(&md);
    let html = format!(
        include_str!("../data/index.html"),
        title = format!("Index of /{}", path.join("/")),
        content = content
    );
    html
}

pub fn md_to_html_gfm_highlight(md: &str) -> String {
    // GitHubっぽい拡張（必要なものだけON）
    let mut opt = Options::default();
    opt.extension.table = true;
    opt.extension.tasklist = true;
    opt.extension.strikethrough = true;
    opt.extension.autolink = true;
    opt.extension.footnotes = true;
    opt.extension.inline_footnotes = true;
    opt.extension.math_dollars = true;
    opt.extension.subscript = true;    // x~2~ :contentReference[oaicite:4]{index=4}
    opt.extension.superscript = true;  // x^2^ :contentReference[oaicite:5]{index=5}
    opt.extension.underline = true;    // __underline__ :contentReference[oaicite:6]{index=6}
    opt.extension.spoiler = true;      // ||spoiler|| :contentReference[oaicite:7]{index=7}
    opt.extension.wikilinks_title_after_pipe = true; // [[Link|Title]] :contentReference[oaicite:9]{index=9}
    opt.extension.highlight = true;    // ==highlight== :contentReference[oaicite:8]{index=8}
    // opt.render.unsafe_ = true; // 生HTMLを許可したいなら

    // ハイライト
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    // ASTでsection挿入 → HTML出力
    let arena = Arena::new();
    let root = parse_document(&arena, md, &opt); // AST作る :contentReference[oaicite:1]{index=1}
    inject_quote_alerts(&arena, root);
    inject_sections(&arena, root);
    inject_mathml(root);

    let mut out = String::new();
    format_html_with_plugins(root, &opt, &mut out, &plugins).unwrap(); // HTML化 :contentReference[oaicite:2]{index=2}
    out
}


/// 見出しノードの子孫から「見える文字列」を抽出する
fn extract_heading_text<'a>(heading: &'a AstNode<'a>) -> String {
    fn walk<'a>(n: &'a AstNode<'a>, out: &mut String) {
        match &n.data.borrow().value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(code) => out.push_str(&code.literal),
            NodeValue::LineBreak | NodeValue::SoftBreak => out.push(' '),
            _ => {}
        }
        for c in n.children() {
            walk(c, out);
        }
    }

    let mut s = String::new();
    for c in heading.children() {
        walk(c, &mut s);
    }

    // 余分な空白をつぶす
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 見出しテキストから「安全な id 用slug」を作る
fn slugify_id_unicode(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in s.chars() {
        // 空白は区切り
        if ch.is_whitespace() {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
            continue;
        }

        // 許可: 文字/数字（Unicode含む） + '_' + '-' 
        // 記号類は '-' に寄せる
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
            prev_dash = false;
        } else {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
        }
    }

    // 前後の '-' を削る
    while out.starts_with('-') { out.remove(0); }
    while out.ends_with('-') { out.pop(); }

    if out.is_empty() { "section".to_string() } else { out }
}

/// HTML属性値のエスケープ（念のため）
/// slugify でだいぶ安全になってるけど、保険で入れる
fn escape_attr(s: &str) -> String {
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

/// 見出しレベルに応じて <section> を入れ子にする + section id を見出し内容から生成（ユニーク化）
pub fn inject_sections<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>) {
    fn collect<'a>(node: &'a AstNode<'a>, out: &mut Vec<&'a AstNode<'a>>) {
        out.push(node);
        for c in node.children() {
            collect(c, out);
        }
    }

    let mut nodes = Vec::new();
    collect(root, &mut nodes);

    let mut stack: Vec<u8> = Vec::new(); // open section の見出しレベル
    let mut used: HashMap<String, u32> = HashMap::new(); // slug -> count

    for &n in &nodes {
        // 見出し判定
        let heading_level = {
            let data = n.data.borrow();
            match &data.value {
                NodeValue::Heading(h) => Some(h.level),
                _ => None,
            }
        };
        let Some(level) = heading_level else { continue };

        // 見出しテキスト → slug
        let title = extract_heading_text(n);
        let base = slugify_id_unicode(&title);

        // ユニーク化: "foo", "foo-2", "foo-3", ...
        let id = match used.get_mut(&base) {
            None => {
                used.insert(base.clone(), 1);
                base
            }
            Some(cnt) => {
                *cnt += 1;
                format!("{}-{}", base, *cnt)
            }
        };

        let id_esc = escape_attr(&id);
        let section_open = format!("<section class=\"md-section\" id=\"{}\">", id_esc);

        // level以上の開いているsectionを閉じる
        while let Some(&top) = stack.last() {
            if top >= level {
                let close = arena.alloc(AstNode::new(RefCell::new(comrak::nodes::Ast::new(
                    NodeValue::Raw("</section>\n".into()),
                    Default::default(),
                ))));
                n.insert_before(close);
                stack.pop();
            } else {
                break;
            }
        }

        // 新しいsectionを開く（見出し直前に挿入）
        let open = arena.alloc(AstNode::new(RefCell::new(comrak::nodes::Ast::new(
            NodeValue::Raw(format!("{}\n", section_open).into()),
            Default::default(),
        ))));
        n.insert_before(open);
        stack.push(level);
    }

    // 文末で残っている section を全部閉じる
    for _ in stack.into_iter().rev() {
        let close = arena.alloc(AstNode::new(RefCell::new(comrak::nodes::Ast::new(
            NodeValue::Raw("</section>\n".into()),
            Default::default(),
        ))));
        root.append(close);
    }
}

/// comrak の Math ノードを MathML に変換して Raw ノードへ置換
fn inject_mathml<'a>(root: &'a AstNode<'a>) {
    // 先にノード列を作る（走査中に書き換えるため）
    fn collect<'a>(n: &'a AstNode<'a>, out: &mut Vec<&'a AstNode<'a>>) {
        out.push(n);
        for c in n.children() {
            collect(c, out);
        }
    }

    let mut nodes = Vec::new();
    collect(root, &mut nodes);

    for &n in &nodes {
        // Math ノードなら変換
        let (latex, display_math) = {
            let data = n.data.borrow();
            match &data.value {
                NodeValue::Math(m) => (m.literal.clone(), m.display_math),
                _ => continue,
            }
        };

        // LaTeX → MathML
        let mut mathml = String::new();
        let storage = Storage::new();
        let ok = push_mathml(&mut mathml, Parser::new(&latex, &storage), RenderConfig::default()).is_ok();

        // 失敗時は安全側に “元のテキスト” を表示（ここは好みで）
        let html = if ok {
            // display math ならブロックに寄せる（MathML自体は <math> を吐く）
            if display_math {
                format!("<div class=\"math math-display\">{}</div>", mathml)
            } else {
                format!("<span class=\"math math-inline\">{}</span>", mathml)
            }
        } else {
            // 変換失敗：そのまま文字として出す（Raw だと注入になるのでエスケープする）
            let escaped = escape_attr(&latex);
            if display_math {
                format!("<div class=\"math math-display\"><code>{}</code></div>", escaped)
            } else {
                format!("<span class=\"math math-inline\"><code>{}</code></span>", escaped)
            }
        };

        // ノード置換（Raw は “そのままHTMLへ”）
        n.data.borrow_mut().value = NodeValue::Raw(html.into());
    }
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
    // s の先頭が "[!TIP]" みたいになってたら (class, 消す文字数) を返す
    // 例: "[!TIP] hello" -> ("quote-tip", 6)
    let s = s.trim_start();
    if !s.starts_with("[!") {
        return None;
    }
    let close = s.find(']')?;
    let kind = &s[2..close]; // "!": は除いた
    let kind = kind.trim().to_ascii_lowercase();
    let cls = alert_kind_class(&kind)?;
    Some((cls, close + 1)) // ']' まで含めて消す
}

/// Paragraph の先頭から marker を削る（同じ段落内に本文がある場合を想定）
fn strip_marker_from_first_paragraph<'a>(para: &'a AstNode<'a>) -> Option<&'static str> {
    // Paragraph の最初の子が Text であるケースをまず狙う（ほとんどこれ）
    let first = para.first_child()?;
    let (cls, cut) = match &first.data.borrow().value {
        NodeValue::Text(t) => {
            let (cls, cut) = parse_marker_prefix(t)?;
            (cls, cut)
        }
        _ => return None,
    };

    // Text ノードの先頭から marker 部分だけ削る
    {
        let mut d = first.data.borrow_mut();
        if let NodeValue::Text(t) = &mut d.value {
            // marker 部分を削除して残りを左詰め
            let after = t.trim_start(); // parse_marker_prefix も trim_start してる前提
            let rest = &after[cut..];
            *t = std::borrow::Cow::Owned(rest.trim_start().to_string());
        }
    }

    // marker の直後が「改行(SoftBreak/LineBreak)」ならそれも消す（> [!TIP]\n> 本文 の場合）
    if let Some(next) = first.next_sibling() {
        let is_break = matches!(next.data.borrow().value, NodeValue::SoftBreak | NodeValue::LineBreak);
        if is_break {
            next.detach();
        }
    }

    // もし Paragraph が空になってたら Paragraph 自体を消す
    let now = para
        .children()
        .filter_map(|c| match &c.data.borrow().value {
            NodeValue::Text(t) => Some(t.to_string()),
            _ => None,
        })
        .collect::<String>()
        .trim()
        .to_string();

    if now.is_empty() {
        para.detach();
    }

    Some(cls)
}

/// BlockQuote を class 付き blockquote に置換（[!TIP] 等）
pub fn inject_quote_alerts<'a>(arena: &'a comrak::Arena<'a>, root: &'a AstNode<'a>) {
    fn collect<'a>(n: &'a AstNode<'a>, out: &mut Vec<&'a AstNode<'a>>) {
        out.push(n);
        for c in n.children() {
            collect(c, out);
        }
    }
    let mut nodes = Vec::new();
    collect(root, &mut nodes);

    for &bq in &nodes {
        if !matches!(bq.data.borrow().value, NodeValue::BlockQuote) {
            continue;
        }

        // 先頭が Paragraph じゃなければ対象外
        let first = match bq.first_child() {
            Some(c) => c,
            None => continue,
        };
        if !matches!(first.data.borrow().value, NodeValue::Paragraph) {
            continue;
        }

        // Paragraph の先頭から marker を剥がして class を得る
        let Some(cls) = strip_marker_from_first_paragraph(first) else { continue };

        // <blockquote class="..."> を自前で作るので、comrak の BlockQuote は剥がす
        let open = arena.alloc(AstNode::new(RefCell::new(comrak::nodes::Ast::new(
            NodeValue::Raw(format!("<blockquote class=\"{}\">\n", cls).into()),
            Default::default(),
        ))));
        bq.insert_before(open);

        let close = arena.alloc(AstNode::new(RefCell::new(comrak::nodes::Ast::new(
            NodeValue::Raw("</blockquote>\n".into()),
            Default::default(),
        ))));
        bq.insert_after(close);

        // bq の子を全部 close の前へ移す（順序維持）
        let children: Vec<_> = bq.children().collect();
        for ch in children {
            ch.detach();
            close.insert_before(ch);
        }

        // 元の BlockQuote を消す
        bq.detach();
    }
}
```