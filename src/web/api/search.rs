
use serde::Serialize;
use tf_idf_vectorizer::Hits;

use crate::{index::{index::DocumentID, search::SearchQuery}, utils::parse_http_like_range, web::context::SiteContextShared};

pub struct SearchAPI;

impl SearchAPI {
    pub async fn search(s_ctx: &SiteContextShared, query: SearchQuery) -> Option<Hits<DocumentID>> {
        s_ctx.index.search(query).await
    }

    pub fn tag_get_all(s_ctx: &SiteContextShared, range_text: &str) -> Vec<TagResult> {
        let mut list = s_ctx.index.tag_nap.load_full().map.iter().enumerate().filter_map(|(id, (name, count))| {
            if *count > 0 {
                Some(TagResult {
                    id: id as u32,
                    name: name.clone(),
                    count: *count,
                })
            } else {
                None
            }
        }).collect::<Vec<_>>();
        list.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        let len = list.len();

        let Some(range) = parse_http_like_range(range_text, len) else {
            return Vec::new();
        };

        list
            .into_iter()
            .skip(range.start)
            .take(range.end.saturating_sub(range.start))
            .collect()
    }
}

#[derive(Serialize)]
pub struct TagResult {
    pub id: u32,
    pub name: String,
    pub count: u32,
}