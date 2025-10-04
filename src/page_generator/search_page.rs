
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time;
use std::time::Duration;

use kurosabi::html_format;
use reqwest::Client;
use kurosabi::kurosabi::Context;

use crate::context::SiteContext;
use crate::api::schema::{ResEntry, SearchApiResult};

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

pub struct SearchPage {
    /// search API endpoint
    /// eg. "localhost:90"
    pub search_api_endpoint: String,
    pub counter: Arc<AtomicU64>,
    client: Client,
}

impl SearchPage {
    pub fn new(search_api_endpoint: &str) -> Self {
        // Keep-Alive 有効な再利用クライアント
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .timeout(Duration::from_secs(2))
            .build()
            .expect("build reqwest client");
        Self {
            search_api_endpoint: search_api_endpoint.to_string(),
            counter: Arc::new(AtomicU64::new(0)),
            client,
        }
    }
    pub async fn generate_search_page(&self, c: &Context<SiteContext>) -> Result<String, u16> {
        const SEARCH_TEMPLATE: &str = include_str!("../../data/pages/search/search_result.html");
        const SEARCH_UI: &str = include_str!("../../data/pages/search/index.html");

        let qs = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        if qs.is_empty() {
            // まず現在保持している値を表示用に取得
            let snapshot = self.counter.load(Ordering::Relaxed);
            let index_size_display = if snapshot == 0 {
                "loading".to_string()
            } else {
                snapshot.to_string()
            };

            // 非同期で最新値を取得して更新（結果は次回アクセス時に反映）
            {
                let endpoint = format!("{}/status", self.search_api_endpoint);
                let counter = Arc::clone(&self.counter);
                let client = self.client.clone();
                tokio::spawn(async move {
                    if let Ok(resp) = client.get(endpoint).send().await {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(n) = json.get("documents").and_then(|v| v.as_u64()) {
                                counter.store(n, Ordering::Relaxed);
                            }
                        }
                    }
                });
            }

            let res = html_format!(SEARCH_UI, index_size = index_size_display);
            return Ok(res);
        }

        let url = format!("{}/search?{}", self.search_api_endpoint, qs);
        let resp = self.client.get(&url).send().await.map_err(|_| 502u16)?;
        let parsed = resp.json::<SearchApiResult>().await.map_err(|_| 502u16)?;

        match parsed {
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
                    request_url = url,
                    tokens = tokenize_query.join(", ")
                );
                Ok(res)
            }
            SearchApiResult::Failed { .. } => Err(500u16),
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