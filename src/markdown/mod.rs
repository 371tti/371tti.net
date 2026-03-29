use std::collections::HashSet;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PageMeta {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    #[serde(default)]
    pub is_complete: bool,
    #[serde(default)]
    pub parse_err: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

impl PageMeta {
    pub fn title(&self) -> String {
        match &self.title {
            Some(t) => t.clone(),
            None => "Untitled".to_string(),
        }
    }

    pub fn authors(&self) -> String {
        match &self.authors {
            Some(a) => a.join(", "),
            None => "Unknown".to_string(),
        }
    }

    pub fn description(&self) -> String {
        match &self.description {
            Some(d) => d.clone(),
            None => "".to_string(),
        }
    }

    pub fn tags(&self) -> Vec<String> {
    let Some(tags) = &self.tags else {
        return Vec::new();
    };

    let mut seen = HashSet::new();

    tags.iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .filter(|t| seen.insert(t.clone()))
        .collect()
    }

    pub fn on_memory_size_hint(&self) -> usize {
        self.title.as_ref().map_or(0, |t| t.capacity() + std::mem::size_of::<String>() + 24)
            + self.authors.as_ref().map_or(0, |a| a.iter().map(|s| s.capacity() + std::mem::size_of::<String>() + 24).sum())
            + self.description.as_ref().map_or(0, |d| d.capacity() + std::mem::size_of::<String>() + 24)
            + self.tags.as_ref().map_or(0, |tags| tags.iter().map(|t| t.capacity() + std::mem::size_of::<String>() + 24).sum())
    }
}
