use std::{hash::Hash, path::Path, sync::Arc};

use arc_swap::ArcSwap;
use half::f16;
use kurosabi::connection::file::FileContentBuilder;
use serde::Serialize;
use tf_idf_vectorizer::{Corpus, TFIDFVectorizer, TermFrequency, utils::datastruct::map::IndexMap};
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::{
    config::Config,
    git::FileChange,
    index::{build_index_plan, tokenizer::SudachiTokenizer},
    markdown::PageMeta,
    utils::Html2Text,
    web::{
        TemplateService,
        api::docs::{DocKind, classify_mime},
    },
};

pub type IndexParameterType = f16;

pub struct Index {
    pub corpus: Arc<Corpus>,
    pub indexes: ArcSwap<TFIDFVectorizer<IndexParameterType, DocumentID>>,
    pub tag_nap: ArcSwap<LinkIDMap<String>>,
    pub config: Arc<Config>,
    pub tokenizer: SudachiTokenizer,
}

#[derive(Clone, Serialize)]
pub struct DocumentID {
    pub tag_ids: Vec<u32>,
    pub path: String,
    pub title: Option<String>,
}

impl Hash for DocumentID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for DocumentID {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for DocumentID {}

impl Index {
    pub async fn new(config: Arc<Config>) -> Self {
        let corpus = Arc::new(Corpus::new());
        let indexes = ArcSwap::new(Arc::new(TFIDFVectorizer::new(corpus.clone())));
        let mut instance = Self {
            corpus,
            indexes,
            tag_nap: ArcSwap::new(Arc::new(LinkIDMap::new())),
            config,
            tokenizer: SudachiTokenizer::new().expect("Failed to initialize SudachiTokenizer"),
        };
        instance.index_all().await;
        instance
    }

    pub async fn index_all(&mut self) {
        let base_dir = Path::new(&self.config.base_dir);
        let rel_paths = Self::collect_index_targets(base_dir).await;
        let mut index = TFIDFVectorizer::new(self.corpus.clone());
        let mut tag_nap = LinkIDMap::new();

        for rel_path in rel_paths {
            if self
                .add(&mut index, rel_path.clone(), &mut tag_nap)
                .await
                .is_some()
            {
                log::info!("Indexed file: {}", rel_path);
            } else {
                log::warn!("Failed to index file: {}", rel_path);
            }
        }
        index.update_idf();
        self.indexes.store(Arc::new(index));
        self.tag_nap.store(Arc::new(tag_nap));
    }

    pub async fn update_index(&self, file_changes: &[FileChange]) {
        log::info!("Updating index with");
        let plan = build_index_plan(file_changes);
        let mut index = self.indexes.load_full();
        let index = Arc::make_mut(&mut index);
        let mut tag_nap = self.tag_nap.load_full().clone();
        let tag_nap = Arc::make_mut(&mut tag_nap);
        for del in &plan.deletes {
            if self.del(index, del.clone(), tag_nap).is_some() {
                log::info!("Deleted index for file: {}", del);
            } else {
                log::warn!("Failed to delete index for file (not found): {}", del);
            }
        }
        for add in &plan.adds {
            if self.add(index, add.clone(), tag_nap).await.is_some() {
                log::info!("Added index for file: {}", add);
            } else {
                log::warn!("Failed to add index for file: {}", add);
            }
        }
        index.update_idf();
        self.indexes.store(Arc::new(index.clone()));
        self.tag_nap.store(Arc::new(tag_nap.clone()));
    }

    async fn add(
        &self,
        index: &mut TFIDFVectorizer<IndexParameterType, DocumentID>,
        path: String,
        tag_ids: &mut LinkIDMap<String>,
    ) -> Option<()> {
        let (meta, content) = self.read_file(&path.split('/').collect::<Vec<_>>()).await?;
        let key = DocumentID {
            tag_ids: Self::register_tags(tag_ids, meta.tags()),
            path,
            title: meta.title.clone(),
        };
        if let Some(tfq) = self.tokenize_wrapper(&meta, &content) {
            index.add_doc(key, &tfq);
            Some(())
        } else {
            None
        }
    }

    fn del(
        &self,
        index: &mut TFIDFVectorizer<IndexParameterType, DocumentID>,
        path: String,
        tag_ids: &mut LinkIDMap<String>,
    ) -> Option<()> {
        let key = DocumentID {
            tag_ids: vec![],
            path,
            title: None,
        };
        if let Some(ind) = index.documents.get_index(&key) {
            let full_key = index.documents.get_key_with_index(ind).unwrap();
            Self::unregister_tags(tag_ids, full_key.tag_ids.clone());
            index.del_doc(&key);
            Some(())
        } else {
            None
        }
    }

    fn register_tags(tag_ids: &mut LinkIDMap<String>, tags: Vec<String>) -> Vec<u32> {
        tags.into_iter().map(|t| tag_ids.add(t)).collect()
    }

    fn unregister_tags(tag_ids: &mut LinkIDMap<String>, tag_id_vec: Vec<u32>) {
        for tag_id in tag_id_vec {
            if let Some(tag) = tag_ids.get(tag_id as usize) {
                let tag_clone = tag.clone();
                tag_ids.sub(&tag_clone);
            }
        }
    }

    async fn read_file(&self, path: &[&str]) -> Option<(PageMeta, String)> {
        let builder = FileContentBuilder::base(&self.config.base_dir)
            .path_url_segs(path)
            .check_file_exists()
            .await
            .ok()?;

        let mut file = builder.build().await.ok()?;
        match classify_mime(&file.mime_type, &file.file_path) {
            DocKind::Markdown => {
                let mut buf = String::new();
                let _bytes = file.file.read_to_string(&mut buf).await.ok()?;
                Some(TemplateService::parse_front_matter(buf, path))
            }
            DocKind::Html => {
                let mut buf = String::new();
                let _bytes = file.file.read_to_string(&mut buf).await.ok()?;
                let (meta, content_html) = TemplateService::parse_front_matter(buf, path);
                let content_text = Html2Text::strip_html_text(&content_html);
                Some((meta, content_text))
            }
            DocKind::Other => None,
        }
    }

    fn tokenize_wrapper(&self, meta: &PageMeta, content: &str) -> Option<TermFrequency> {
        let mut tfq = TermFrequency::new();
        let mut buf = String::new();
        if let Some(tags) = &meta.authors {
            buf.push_str(&tags.join(" "));
        }
        if let Some(tags) = &meta.tags {
            buf.push_str(&tags.join(" "));
        }
        if let Some(title) = &meta.title {
            buf.push_str(title);
        }
        if let Some(description) = &meta.description {
            buf.push_str(description);
        }
        buf.push_str(content);
        let tokens = self.tokenizer.mix_doc_tokenizer(&buf).ok()?;
        tfq.add_terms(&tokens.0);
        Some(tfq)
    }

    async fn collect_index_targets(base_dir: &Path) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = vec![base_dir.to_path_buf()];

        while let Some(dir) = stack.pop() {
            let mut entries = match fs::read_dir(&dir).await {
                Ok(v) => v,
                Err(err) => {
                    log::warn!("Failed to read directory {}: {}", dir.display(), err);
                    continue;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with('.'))
                {
                    continue;
                }
                let file_type = match entry.file_type().await {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                if file_type.is_dir() {
                    stack.push(path);
                    continue;
                }

                if !file_type.is_file() {
                    continue;
                }

                if let Ok(rel) = path.strip_prefix(base_dir) {
                    let normalized = rel
                        .to_string_lossy()
                        .replace('\\', "/")
                        .trim_start_matches('/')
                        .to_string();
                    out.push(normalized);
                }
            }
        }

        out.sort_unstable();
        out
    }
}

#[derive(Clone)]
pub struct LinkIDMap<T: Eq + std::hash::Hash + Clone> {
    /// T to RefCount map
    pub map: IndexMap<T, u32>,
}

impl<T: Eq + std::hash::Hash + Clone> LinkIDMap<T> {
    pub fn new() -> Self {
        LinkIDMap {
            map: IndexMap::new(),
        }
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        let (key, ref_count) = self.map.get_key_value_with_index(id)?;
        (*ref_count > 0).then_some(key)
    }

    pub fn add(&mut self, item: T) -> u32 {
        if let Some(ind) = self.map.get_index(&item) {
            let e = self.map.get_with_index_mut(ind).unwrap();
            *e += 1u32;
            ind as u32
        } else {
            let ref_count_vec = self.map.values();
            if let Some(first_zero_ind) = ref_count_vec.iter().position(|&c| c == 0) {
                let key = self.map.get_key_with_index(first_zero_ind).unwrap().clone();
                self.map.insert(item, 1u32);
                self.map.swap_remove(&key);
                first_zero_ind as u32
            } else {
                let ind = self.map.len();
                self.map.insert(item, 1u32);
                ind as u32
            }
        }
    }

    pub fn sub(&mut self, item: &T) -> Option<u32> {
        if let Some(ind) = self.map.get_index(item) {
            let e = self.map.get_with_index_mut(ind).unwrap();
            if *e > 0 {
                *e -= 1;
                Some(ind as u32)
            } else {
                None
            }
        } else {
            None
        }
    }
}
