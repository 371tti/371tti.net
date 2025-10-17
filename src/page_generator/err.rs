use std::collections::HashMap;
use kurosabi::{html_format, kurosabi::Context};
use serde::Deserialize;
use crate::context::SiteContext;

const ERR_TEMPLATE: &'static str = include_str!("../../data/pages/err/index.html");

pub struct ErrPage {
    pub err_infos: HashMap<String, StatusCodeInfo>,
}

impl ErrPage {
    pub fn new() -> Self {
        let raw_err_infos = serde_json::from_str(include_str!("../../data/pages/err/status_code.json")).unwrap();
        let err_infos: HashMap<String, StatusCodeInfo> = serde_json::from_value(raw_err_infos).unwrap();

        Self {
            err_infos: err_infos,
        }
    }

    pub fn status_page(mut c: Context<SiteContext>, code: u16, adv_message: &str) -> Context<SiteContext> {
        let binding = StatusCodeInfo::default();
        let code_info = c.c.ssr.err_page.err_infos.get(&code.to_string()).unwrap_or(&binding);
        let mut solutions = String::new();
        let mut debug_info = String::new();
        for (i, solution) in code_info.suggest.iter().enumerate() {
            solutions.push_str(&format!("<li>{}. {}</li>", i + 1, solution));
        }
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Accept-Encoding", c.req.header.get("Accept-Encoding").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "User-Agent", c.req.header.get("User-Agent").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Host", c.req.header.get("Host").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Accept-Language", c.req.header.get("Accept-Language").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Accept", c.req.header.get("Accept").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Path", c.req.path.path));
        debug_info.push_str(&format!("<li>{}: <ul><li>{}</li></ul></li>", "Connection", c.req.header.get("Connection").unwrap_or(&"None".to_string())));
        debug_info.push_str(&format!(
            "<li>{}: <ul><li>{}</li></ul></li>",
            "TimeStamp",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        ));
        let message = if adv_message.is_empty() { code_info.message.clone() } else { code_info.message.clone() + " - " + adv_message };
        c.res.set_status(code);
        let html = html_format!(
            ERR_TEMPLATE,
            color = code_info.color,
            code = c.res.code,
            ms = message,
            solution = solutions,
            debug = debug_info,
        );
        c.res.html(&html);
        c
    }
}

#[derive(Deserialize)]
pub struct StatusCodeInfo {
    pub color: String,
    pub message: String,
    pub suggest: Vec<String>,
}

impl Default for StatusCodeInfo {
    fn default() -> Self {
        Self {
            color: "#00000000".to_string(),
            message: "Unknown Error".to_string(),
            suggest: vec!["Please contact support.".to_string()],
        }
    }
    
}