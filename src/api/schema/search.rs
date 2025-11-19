use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tf_idf_vectorizer::TokenFrequency;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub url: String,
    pub score: f64,
    pub length: u64,
    pub description: Option<String>,
    pub title: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaReq {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum MetaRes {
    #[serde(rename = "true")]
    Success {
        meta: IndexMeta,
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenFreqReq {
    pub url: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum TokenFreqRes {
    #[serde(rename = "true")]
    Success {
        fq: TokenFrequency,
    },
    #[serde(rename = "false")]
    Failed {
        error: String,
    },
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
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub descriptions: Option<String>,
    /// クロール時のターゲットセレクタ
    /// wikipediaなら .mw-body-content など
    #[serde(default)]
    pub target_selector: Option<String>,
}


/// Index の基本情報
/// URL, title, description, favicon, time, points, tags
/// Hash と Equal は URL のみで判定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMeta {
    pub id: usize,
    /// token_sum
    pub token_sum: u64,
    /// URL
    /// only URL is used for Hash and Equal
    pub url: Box<str>,
    /// Title
    pub title: Box<str>,
    /// Description
    pub description: Box<str>,
    /// links
    pub links: Vec<Box<str>>,
    /// Favicon URL
    pub favicon: Option<Box<str>>,
    /// language code (e.g. "en", "ja", etc)
    pub lang: Option<Box<str>>,
    /// Upload Time
    pub time: DateTime<Utc>,
    /// Score
    pub points: f64,
    /// Tags
    /// General Tag:
    /// - Wiki: wikipedia, ニコニコ大百科, etc
    /// - News: yahoo!, GIGAZINE, ITmedia, etc
    /// - SNS: twitter, facebook, youtube, instagram, etc
    /// - Blog: hatena, zenn, etc
    /// - Forum: 5ch, reddit, stackoverflow, etc
    /// - Shopping: amazon, rakuten, ebay, etc
    /// - Academic: arxiv, ciNii, etc
    /// - Tools: translate, map, etc
    pub tags: Tags,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tags(u64);

impl Tags {
    pub const WIKI: u64 = 1 << 0;
    pub const NEWS: u64 = 1 << 1;
    pub const SNS: u64 = 1 << 2;
    pub const BLOG: u64 = 1 << 3;
    pub const FORUM: u64 = 1 << 4;
    pub const SHOPPING: u64 = 1 << 5;
    pub const ACADEMIC: u64 = 1 << 6;
    pub const TOOLS: u64 = 1 << 7;

    /// eg: Tags::new(Tags::NEWS | Tags::BLOG)
    pub fn new(set: u64) -> Self {
        Self(set)
    }

    /// すべて満たしてるか
    /// tagはselfに含まれている必要がある
    /// eg: is_filter_contains(Tags::NEWS | Tags::BLOG) -> NEWSとBLOGの両方を含む場合のみtrue
    pub fn is_filter_contains<T: Into<u64> + Copy>(&self, tag: T) -> bool {
        (self.0 & tag.into()) == tag.into()
    }

    pub fn contains<T: Into<u64>>(&self, tag: T) -> bool {
        (self.0 & tag.into()) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn tags(&self) -> Vec<Box<str>> {
        let mut result = Vec::new();
        if self.contains(Self::WIKI) { result.push("WIKI".into()); }
        if self.contains(Self::NEWS) { result.push("NEWS".into()); }
        if self.contains(Self::SNS) { result.push("SNS".into()); }
        if self.contains(Self::BLOG) { result.push("BLOG".into()); }
        if self.contains(Self::FORUM) { result.push("FORUM".into()); }
        if self.contains(Self::SHOPPING) { result.push("SHOPPING".into()); }
        if self.contains(Self::ACADEMIC) { result.push("ACADEMIC".into()); }
        if self.contains(Self::TOOLS) { result.push("TOOLS".into()); }
        result
    }

    pub fn from_strs<T>(tags: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        let mut set = 0;
        for tag in tags {
            let s = tag.as_ref();
            if s.eq_ignore_ascii_case("wiki")      { set |= Self::WIKI; }
            else if s.eq_ignore_ascii_case("news") { set |= Self::NEWS; }
            else if s.eq_ignore_ascii_case("sns")  { set |= Self::SNS; }
            else if s.eq_ignore_ascii_case("blog") { set |= Self::BLOG; }
            else if s.eq_ignore_ascii_case("forum"){ set |= Self::FORUM; }
            else if s.eq_ignore_ascii_case("shopping"){ set |= Self::SHOPPING; }
            else if s.eq_ignore_ascii_case("academic"){ set |= Self::ACADEMIC; }
            else if s.eq_ignore_ascii_case("tools"){ set |= Self::TOOLS; }
        }
        Self(set)
    }
}

impl PartialEq for IndexMeta {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Into<u64> for Tags {
    fn into(self) -> u64 {
        self.0
    }
}