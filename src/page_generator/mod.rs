use kurosabi::{html_format, kurosabi::Context};
use regex::Regex;

use crate::context::SiteContext;

pub mod err;
pub mod search;

pub struct PageGenerator {
    pub err_page: err::ErrPage,
    pub search_page: search::SearchPage,
}

impl PageGenerator {
    pub fn new(search_api_endpoint: &str) -> Self {
        Self {
            err_page: err::ErrPage::new(),
            search_page: search::SearchPage::new(search_api_endpoint),
        }
    }

    /// htnlページの基本生成メソッド
    pub fn base(mut c: Context<SiteContext>, html: &str, name: &str, image_url: Option<&str>) -> Context<SiteContext> {
        
        let m_html = html_format!(
            include_str!("../../data/pages/base.html"),
            body = html,
            name = name,
            url = c.req.path.get_raw_path(),
            image_url = image_url.unwrap_or("/banner.gif"),
        );

        if c.req.header.get_user_agent().map_or(false, |ua| ua.contains("curl")) {
            let md = html_to_md(m_html);
            c.res.text(&md);
            c.res.header.set("Content-Type", "text/markdown; charset=utf-8");
            return c;
        }
        c.res.html(&m_html);
        c
    }
}

pub fn html_to_md(html: String) -> String {
    // まずはタグを Markdown にざっくり変換
    let mut s = html;


    // ------------ 0. ノイズ削除 --------------

    // <script> / <style> / <head> / <noscript> を中身ごと削除
    let re_script = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    s = re_script.replace_all(&s, "").to_string();

    let re_style = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    s = re_style.replace_all(&s, "").to_string();

    let re_head = Regex::new(r"(?is)<head[^>]*>.*?</head>").unwrap();
    s = re_head.replace_all(&s, "").to_string();

    let re_noscript = Regex::new(r"(?is)<noscript[^>]*>.*?</noscript>").unwrap();
    s = re_noscript.replace_all(&s, "").to_string();

    // data-code="...<a href=...>..." を削除（バナーの複製を防ぐ）
    let re_data_code_sq = Regex::new(r"(?is)\sdata-code='[^']*'").unwrap();
    s = re_data_code_sq.replace_all(&s, "").to_string();
    let re_data_code_dq = Regex::new(r#"(?is)\sdata-code="[^"]*""#).unwrap();
    s = re_data_code_dq.replace_all(&s, "").to_string();

    // ------------ 1. HTML → Markdown ざっくり変換 --------------

    // <br> 系
    s = s
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");

    // 見出し <h1>〜<h6>
    for i in 1..=6 {
        let re = Regex::new(&format!(r"(?is)<h{0}[^>]*>(.*?)</h{0}>", i)).unwrap();
        s = re
            .replace_all(&s, format!("{} $1\n\n", "#".repeat(i)))
            .to_string();
    }

    // strong / b
    let re = Regex::new(r"(?is)<(strong|b)>(.*?)</(strong|b)>").unwrap();
    s = re.replace_all(&s, "**$2**").to_string();

    // em / i
    let re = Regex::new(r"(?is)<(em|i)>(.*?)</(em|i)>").unwrap();
    s = re.replace_all(&s, "*$2*").to_string();

    // 行内コード
    let re = Regex::new(r"(?is)<code>(.*?)</code>").unwrap();
    s = re.replace_all(&s, "`$1`").to_string();

    // 複数行コード <pre>〜</pre>
    let re = Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").unwrap();
    s = re.replace_all(&s, "```\n$1\n```\n\n").to_string();

    // 1-1. <a><img alt="..."></a> 専用処理（バナー）
    //   → [alt](href) にする
    let re_link_img = Regex::new(
        r#"(?is)<a\s+[^>]*href="([^"]+)"[^>]*>\s*<img\s+[^>]*alt="([^"]+)"[^>]*>\s*</a>"#
    ).unwrap();
    s = re_link_img
        .replace_all(&s, "[$2]($1)\n")
        .to_string();

    // 1-2. 通常の <a href="...">text</a>
    let re_link = Regex::new(r#"(?is)<a\s+[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).unwrap();
    s = re_link.replace_all(&s, "[$2]($1)").to_string();

    // 段落 <p>〜</p>
    let re = Regex::new(r"(?is)<p[^>]*>(.*?)</p>").unwrap();
    s = re.replace_all(&s, "$1\n\n").to_string();

    // ------------ 2. 残りのタグを全部削除 --------------

    let re_tags = Regex::new(r"(?is)</?[^>]+>").unwrap();
    s = re_tags.replace_all(&s, "").to_string();

    // ------------ 3. Markdown 整形 --------------

    cleanup_markdown(&s)
}

fn cleanup_markdown(input: &str) -> String {
    // まずは、既存のリンク整形
    let mut s = input.to_string();

    // [\n text \n](url) のように分裂したリンクを 1 行にまとめる
    let re_broken_link = Regex::new(
        r"(?m)\[\s*\n\s*([^\n]+?)\s*\n\s*\]\(([^)]+)\)"
    ).unwrap();
    s = re_broken_link
        .replace_all(&s, "[$1]($2)")
        .to_string();

    // リンク末尾のゴミ `)'>` / `)">` を除去
    let re_trailing_junk = Regex::new(r#"\)\s*['"]?>"#).unwrap();
    s = re_trailing_junk.replace_all(&s, ")").to_string();

    // ここから「段落まとめ」ロジック
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_paragraph = false;

    for line in s.lines() {
        let trimmed = line.trim();

        // 空行
        if trimmed.is_empty() {
            if in_paragraph {
                // 段落の中の空行は、スペースとして扱って行を続けるだけ
                if !buf.ends_with(' ') {
                    buf.push(' ');
                }
            } else {
                // 段落外の空行は、セクションの区切りとしてそのまま保持
                if !out.last().map(|l| l.is_empty()).unwrap_or(false) {
                    out.push(String::new());
                }
            }
            continue;
        }

        // 行の種別判定
        let is_heading = trimmed.starts_with('#');
        let starts_digit_dot = {
            let mut chars = trimmed.chars();
            let first_is_digit = chars.next().map(|c| c.is_ascii_digit()).unwrap_or(false);
            first_is_digit && chars.as_str().starts_with(". ")
        };
        let starts_list =
            trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("> ")
            || trimmed.starts_with("```");
        let starts_link = trimmed.starts_with('[');

        // 「普通の文章行」かどうか
        let is_plain_text =
            !is_heading &&
            !starts_digit_dot &&
            !starts_list &&
            !starts_link;

        if is_plain_text {
            // 段落内の文章としてまとめる対象
            if !in_paragraph {
                // 新しい段落を開始
                in_paragraph = true;
                if !buf.is_empty() {
                    // 理論上ここには来ないはずだけど念のため
                    out.push(buf.trim().to_string());
                    buf.clear();
                }
            }
            if !buf.is_empty() && !buf.ends_with(' ') {
                buf.push(' ');
            }
            buf.push_str(trimmed);
        } else {
            // 見出し・リスト・リンクなどの構造行
            if in_paragraph && !buf.is_empty() {
                // それまで貯めていた段落をフラッシュ
                out.push(buf.trim().to_string());
                buf.clear();
            }
            in_paragraph = false;

            // 見出しの前には 1 行空ける（直前が空でなければ）
            if is_heading {
                if !out.is_empty() && !out.last().unwrap().is_empty() {
                    out.push(String::new());
                }
            }

            out.push(trimmed.to_string());
        }
    }

    // 最後の段落をフラッシュ
    if in_paragraph && !buf.is_empty() {
        out.push(buf.trim().to_string());
    }

    // 最後に、連続空行を 1 行に圧縮
    let mut final_lines = Vec::new();
    let mut last_empty = false;

    for line in out {
        let is_empty = line.trim().is_empty();
        if is_empty {
            if last_empty {
                continue;
            }
            final_lines.push(String::new());
        } else {
            final_lines.push(line);
        }
        last_empty = is_empty;
    }

    final_lines.join("\n").trim().to_string()
}
