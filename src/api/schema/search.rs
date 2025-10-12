use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub url: String,
    pub score: f64,
    pub length: u64,
    pub description: Option<String>,
    pub title: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum SearchApiResult {
    #[serde(rename = "true")]
    Success {
        query: String,
        tokenize_query: Vec<String>,
        algorithm: String,
        range: Range<usize>,
        results: Vec<ResEntry>,
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResEntry {
    pub url: Box<str>,
    pub title: Box<str>,
    pub favicon: Option<Box<str>>,
    pub tags: Vec<Box<str>>,
    pub descriptions: Box<str>,
    pub score: f64,
    pub point: f64,
    pub length: u64,
    pub id: usize,
    pub index_id: usize,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexReq {
    pub url: String,
    pub title: Option<String>,
    pub favicon: Option<String>,
    /// タグは空でも良い
    /// 例: ["wiki", "blog"]
    /// 使用可能なタグ:
    /// - "wiki": ウィキペディアなどの百科事典
    /// - "news": ニュースサイト
    /// - "sns": ソーシャルメディア
    /// - "blog": ブログ
    /// - "forum": フォーラム
    /// - "shopping": ショッピングサイト
    /// - "academic": 学術論文
    /// - "tools": ツール系サイト
    pub tags: Vec<String>,
    pub descriptions: Option<String>,
}