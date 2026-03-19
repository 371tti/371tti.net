use serde::Deserialize;
use tf_idf_vectorizer::{Hits, Query, SimilarityAlgorithm, TermFrequency};

use crate::index::index::{DocumentID, Index};

#[derive(Clone, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    boolean_mode: bool,
    #[serde(default = "default_similarity_algorithm")]
    algorithm: SimilarityAlgorithm,
    query: String,
    #[serde(default)]
    tags: Vec<u32>,
    #[serde(default)]
    order: ResultOrder,
}

fn default_similarity_algorithm() -> SimilarityAlgorithm {
    SimilarityAlgorithm::CosineSimilarity
}

#[derive(Clone, Deserialize)]
pub enum ResultOrder {
    ScoreDescWithTitle,
    ScoreDesc,
}

impl Default for ResultOrder {
    fn default() -> Self {
        Self::ScoreDescWithTitle
    }
}

impl Index {
    pub async fn search(&self, query: SearchQuery) -> Option<Hits<DocumentID>> {
        let mut hits = if query.boolean_mode {
            Some(self.search_boolean(&query))
        } else {
            self.search_fq(&query)
        }?;
        self.filter_by_tags(&mut hits, &query.tags);
        match query.order {
            ResultOrder::ScoreDescWithTitle => {
                hits.list.sort_by(|a, b| {
                    a.key.title.cmp(&b.key.title).then_with(|| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                });
                Some(hits)
            }
            ResultOrder::ScoreDesc => {
                hits.sort_by_score_desc();
                Some(hits)
            }
        }
    }

    pub fn filter_by_tags(&self, hits: &mut Hits<DocumentID>, tags: &[u32]) {
        hits.list.retain(|hit| {
            let hit_tags = &hit.key.tag_ids;
            tags.iter().all(|t| hit_tags.contains(t))
        });
    }

    pub fn search_boolean(&self, query: &SearchQuery) -> Hits<DocumentID> {
        let q = QueryBuilder::new(&query.query).build().unwrap_or_else(|e| {
            log::error!("Failed to parse query '{}': {}", query.query, e);
            Query::none()
        });

        self.indexes
            .load_full()
            .search_uncheck_idf(&query.algorithm, q)
    }

    pub fn search_fq(&self, query: &SearchQuery) -> Option<Hits<DocumentID>> {
        let terms = self.tokenizer.mix_query_tokenizer(&query.query).ok()?;
        let mut freq = TermFrequency::new();
        freq.add_terms(&terms);
        let q = Query::from_freq_or(&freq);
        Some(
            self.indexes
                .load_full()
                .search_uncheck_idf(&query.algorithm, q),
        )
    }
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    tokens: Vec<QueryToken>,
    pos: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryToken {
    LBracket,
    RBracket,
    And,
    Or,
    Not,
    Word(String),
}

#[derive(Debug)]
struct QueryParseError(String);

impl std::fmt::Display for QueryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query parse error: {}", self.0)
    }
}

impl std::error::Error for QueryParseError {}

impl QueryBuilder {
    pub fn new(query: &str) -> Self {
        Self {
            tokens: Self::tokenize(query),
            pos: 0,
        }
    }

    /// Parse the query string into `tf_idf_vectorizer::Query`.
    ///
    /// Supported:
    /// - `&` AND
    /// - `|` OR
    /// - `!` NOT (prefix)
    /// - `[...]` grouping
    ///
    /// Also supported:
    /// - whitespace between terms is treated as AND
    ///   e.g. `[token0 token1]` == `[token0 & token1]`
    ///
    /// Precedence: `!` > (implicit/explicit `&`) > `|`
    pub fn build(mut self) -> Result<Query, Box<dyn std::error::Error + Send + Sync>> {
        if self.tokens.is_empty() {
            return Ok(Query::none());
        }

        let q = self.parse_or()?;

        if self.pos != self.tokens.len() {
            return Err(Box::new(QueryParseError(format!(
                "unexpected token at end: {:?}",
                self.tokens.get(self.pos)
            ))));
        }

        Ok(q)
    }

    fn tokenize(query: &str) -> Vec<QueryToken> {
        let mut out = Vec::new();
        let mut buf = String::new();

        let flush_word = |out: &mut Vec<QueryToken>, buf: &mut String| {
            if !buf.is_empty() {
                out.push(QueryToken::Word(std::mem::take(buf)));
            }
        };

        for ch in query.chars() {
            match ch {
                // whitespace ends a word (implicit AND is handled in parsing)
                c if c.is_whitespace() => {
                    flush_word(&mut out, &mut buf);
                }
                '[' => {
                    flush_word(&mut out, &mut buf);
                    out.push(QueryToken::LBracket);
                }
                ']' => {
                    flush_word(&mut out, &mut buf);
                    out.push(QueryToken::RBracket);
                }
                '&' => {
                    flush_word(&mut out, &mut buf);
                    out.push(QueryToken::And);
                }
                '|' => {
                    flush_word(&mut out, &mut buf);
                    out.push(QueryToken::Or);
                }
                '!' => {
                    flush_word(&mut out, &mut buf);
                    out.push(QueryToken::Not);
                }
                _ => buf.push(ch),
            }
        }

        flush_word(&mut out, &mut buf);
        out
    }

    fn peek(&self) -> Option<&QueryToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<QueryToken> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: QueryToken) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let got = self.next();
        if got.as_ref() == Some(&want) {
            Ok(())
        } else {
            Err(Box::new(QueryParseError(format!(
                "expected {:?}, got {:?}",
                want, got
            ))))
        }
    }

    // Lowest precedence
    fn parse_or(&mut self) -> Result<Query, Box<dyn std::error::Error + Send + Sync>> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(QueryToken::Or)) {
            self.next();
            let right = self.parse_and()?;
            left = Query::or(left, right);
        }
        Ok(left)
    }

    // AND precedence (explicit '&' or implicit by whitespace / adjacency)
    fn parse_and(&mut self) -> Result<Query, Box<dyn std::error::Error + Send + Sync>> {
        let mut left = self.parse_not()?;

        loop {
            match self.peek() {
                Some(QueryToken::And) => {
                    self.next(); // explicit AND
                    let right = self.parse_not()?;
                    left = Query::and(left, right);
                }
                // implicit AND: next token starts an expression
                Some(QueryToken::Word(_)) | Some(QueryToken::LBracket) | Some(QueryToken::Not) => {
                    let right = self.parse_not()?;
                    left = Query::and(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // Highest precedence (prefix unary)
    fn parse_not(&mut self) -> Result<Query, Box<dyn std::error::Error + Send + Sync>> {
        if matches!(self.peek(), Some(QueryToken::Not)) {
            self.next();
            let inner = self.parse_not()?;
            Ok(Query::not(inner))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Query, Box<dyn std::error::Error + Send + Sync>> {
        match self.next() {
            Some(QueryToken::Word(mut w)) => {
                // allow quoted tokens: "token" or 'token'
                if w.len() >= 2 {
                    let bytes = w.as_bytes();
                    let first = bytes[0];
                    let last = bytes[bytes.len() - 1];
                    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
                        w = w[1..w.len() - 1].to_string();
                    }
                }

                if w.is_empty() {
                    Ok(Query::none())
                } else if w == "*" {
                    Ok(Query::all())
                } else {
                    Ok(Query::term(&w))
                }
            }
            Some(QueryToken::LBracket) => {
                let inner = self.parse_or()?;
                self.expect(QueryToken::RBracket)?;
                Ok(inner)
            }
            Some(tok) => Err(Box::new(QueryParseError(format!(
                "unexpected token: {:?}",
                tok
            )))),
            None => Ok(Query::none()),
        }
    }
}
