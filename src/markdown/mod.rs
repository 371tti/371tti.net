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
}