use std::time::Duration;

use kurosabi::html_format;
use reqwest::Client;
use kurosabi::kurosabi::Context;

use crate::context::SiteContext;
use crate::api::schema::search::{ResEntry, SearchApiResult};
use crate::page_generator::err::ErrPage;

const SEARCH_TEMPLATE: &str = include_str!("../../data/pages/search/search_result.html");
const SEARCH_UI: &str = include_str!("../../data/pages/search/index.html");

pub struct SearchPage {
    /// search API endpoint
    /// eg. "localhost:90"
    pub search_api_endpoint: String,
    client: Client,
}

impl SearchPage {
    pub fn new(search_api_endpoint: &str) -> Self {
        // Keep-Alive 有効な再利用クライアント
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .timeout(Duration::from_secs(5))
            .build()
            .expect("build reqwest client");
        Self {
            search_api_endpoint: search_api_endpoint.to_string(),
            client,
        }
    }

    pub async fn page(mut c: Context<SiteContext>) -> Context<SiteContext> {
        let qs = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        match qs {
            "" => {
                match c.c.ssr.search_page.search_api_status().await {
                    Ok(n) => {
                        c.res.html(&html_format!(SEARCH_UI, index_size = n));
                        c
                    },
                    Err(e) => {
                        ErrPage::status_page(c, e, "Search API unreachable")
                    },
                }
            }
            _ => {
                match c.c.ssr.search_page.search_api(qs).await {
                    Ok(res) => {
                        match res {
                            SearchApiResult::Success { query, tokenize_query, algorithm, range, results } => {
                                let entries = results.into_iter()
                                    .map(|e| Self::generate_entry(e))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                let res = html_format!(
                                    SEARCH_TEMPLATE,
                                    query = query,
                                    algorithm = algorithm,
                                    range = format!("{}..{}", range.start, range.end),
                                    results = entries,
                                    request_url = format!("{}/search?{}", c.c.ssr.search_page.search_api_endpoint, qs),
                                    tokens = tokenize_query.join(", ")
                                );
                                c.res.html(&res);
                                c
                            }
                            SearchApiResult::Failed { error } => {
                                ErrPage::status_page(c, 500u16, &format!("Search API error: {}", error))
                            }
                        }
                    },
                    Err(e) => {
                        return ErrPage::status_page(c, e, "Search API unreachable");
                    },
                }
                
            }
        }
        
    }

    pub async fn search_api(&self, query: &str) -> Result<SearchApiResult, u16> {
        let url = format!("{}/search?{}", self.search_api_endpoint, query);
        let resp = self.client.get(&url).send().await.map_err(|_| 502u16)?;
        let parsed = resp.json::<SearchApiResult>().await.map_err(|_| 502u16)?;
        Ok(parsed)
    }

    pub async fn search_api_status(&self) -> Result<u64, u16> {
        let url = format!("{}/status", self.search_api_endpoint);
        let resp = self.client.get(&url).send().await.map_err(|_| 502u16)?;
        let parsed = resp.json::<serde_json::Value>().await.map_err(|_| 502u16)?;
        if let Some(n) = parsed.get("documents").and_then(|v| v.as_u64()) {
            Ok(n)
        } else {
            Err(502u16)
        }
    }

    pub fn generate_entry(entry: ResEntry) -> String {
        let safe_url = escape_html(&entry.url);
        let safe_title = escape_html(&entry.title);
        let description = format!("<p class=\"description\">{}</p>", escape_html(&entry.descriptions));
        let favicon = match entry.favicon {
            Some(ref fav) => format!(r#"<img height="30" loading="lazy" src="{}" alt="Favicon"> "#, escape_html(fav)),
            None => "".to_string(),
        };
        format!(
            r#"<div class="indent">
                <h3>{favicon}<a href="{url}" target="_blank" rel="noopener noreferrer">{title}</a></h3>
                <p class="score">score: {score}</p>
                <p class="tags">tags: {tags}</p>
                <p><small class="debug">length: {length} tokens, point: {point}, id: {id}, index_id: {index_id}, time: {time}</small></p>
                <p><small class="url">{url}</small></p>
                <div class="pd"></div>
                {description}
            </div>"#,
            favicon = favicon,
            url = safe_url,
            title = safe_title,
            score = entry.score,
            length = entry.length,
            point = entry.point,
            id = entry.id,
            index_id = entry.index_id,
            time = entry.time.to_rfc3339(),
            tags = entry.tags.iter()
                .map(|t| format!(r#"<span class="tag tag-sm">{}</span>"#, escape_html(t)))
                .collect::<Vec<_>>()
                .join(" "),
            description = description,
        )
    }
}

// シンプルなHTMLエスケープ (&, <, >, ", ')
fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
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