use std::sync::Arc;

use kurosabi::utils::url_decode_fast;
use tf_idf_vectorizer::{Corpus, TermFrequency};

use crate::{index::index::DocumentID, web::context::SiteContextShared};

pub struct AnalyzeAPI;

impl AnalyzeAPI {
    pub fn term_freq(s_ctx: &SiteContextShared, path: &[&str]) -> Option<TermFrequency> {
        let joined_path = path.join("/");
        let decoded_path = url_decode_fast(&joined_path).to_string();
        let key = DocumentID {
            tag_ids: Vec::new(),
            path: decoded_path,
            title: None,
        };
        s_ctx.index.indexes.load_full().get_tf_into_term_freq(&key)
    }

    pub fn corpus_freq(s_ctx: &SiteContextShared) -> Arc<Corpus> {
        s_ctx.index.corpus.clone()
    }
}