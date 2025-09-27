
use std::time;

use kurosabi::html_format;
use kurosabi::kurosabi::Context;
use mongodb::bson::DateTime;

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
}

impl SearchPage {
    pub fn new(search_api_endpoint: &str) -> Self {
        Self {
            search_api_endpoint: search_api_endpoint.to_string(),
        }
    }
    pub async fn generate_search_page(&self, c: &Context<SiteContext>) -> Result<String, u16> {
        const SEARCH_TEMPLATE: &str = include_str!("../../data/pages/search/search_result.html");
        const SEARCH_UI: &str = include_str!("../../data/pages/search/index.html");

        let qs = c.req.path.path.splitn(2, '?').nth(1).unwrap_or("");
        if qs.is_empty() {
            // No query string: show search UI
            let index_size = match reqwest::get(format!("{}/status", self.search_api_endpoint)).await {
                Ok(resp) => resp.json::<serde_json::Value>().await
                    .ok()
                    .and_then(|json| json.get("documents").and_then(|v| v.as_u64()))
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                Err(_) => "null".to_string(),
            };
            let res = html_format!(SEARCH_UI, index_size = index_size);
            return Ok(res);
        }

        let url = format!("{}/search?{}", self.search_api_endpoint, qs);
        let resp = reqwest::get(&url).await.map_err(|_| 502u16)?;
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
                <p class="url">{url}</p>
                <p class="score">score: {score}</p>
                <p class="debug">length: {length} tokens, point: {point}, id: {id}, index_id: {index_id}</p>
                <p class="tags">tags: {tags}</p>
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
            tags = entry.tags.iter().map(|t| escape_html(t)).collect::<Vec<_>>().join(", "),
            description = description,
        )
    }
}