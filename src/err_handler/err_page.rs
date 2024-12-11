use std::collections::HashMap;
use actix_web::{body::BoxBody, dev::ServiceResponse, HttpResponse};
use tera::{Tera, Context};
use chrono::Utc;

pub struct ErrHandler {
    pub status_color: HashMap<u16, String>,
    pub status_message: HashMap<u16, String>,
    pub suggestion_fix_message: HashMap<u16, HashMap<u16, String>>,
    pub err_page_template: Tera,
}

impl ErrHandler {

    pub async fn new(templates: Tera) -> Self {
 
        



        ErrHandler {
            status_color,
            status_message,
            suggestion_fix_message,
            err_page_template: templates,
        }
    }

    pub fn page_generate<B>(&self, res: &ServiceResponse<B>) -> HttpResponse<BoxBody> {
        // ステータスコードを取得
        let status_code = res.status().as_u16();

        // ステータスメッセージを取得
        let status_message = self.status_message.get(&status_code)
            .cloned()
            .unwrap_or_else(|| "Unknown Error".to_string());

        // ステータスコードに対応する色を取得
        let status_color = self.status_color.get(&(status_code / 100))
            .cloned()
            .unwrap_or_else(|| "#ffffff".to_string());

        // 提案メッセージを取得
        let suggestions = self.suggestion_fix_message.get(&status_code);
        let suggestion_list: Vec<String> = if let Some(suggestions_map) = suggestions {
            suggestions_map.values().cloned().collect()
        } else {
            Vec::new()
        };

        // デバッグ情報を作成
        let mut debug_info = HashMap::new();
        // Host, Path, Connection, User-Agent, Last-Time, Cf-Connecting-Ip, Accept-Encoding, Accept-Languageなどのヘッダー情報を追加
        debug_info.insert("Host".to_string(),
            res.request().headers().get("Host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );
        debug_info.insert("Path".to_string(), res.request().path().to_string());
        debug_info.insert("Connection".to_string(),
            res.request().headers().get("Connection")
                .and_then(|c| c.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );
        debug_info.insert("User-Agent".to_string(),
            res.request().headers().get("User-Agent")
                .and_then(|ua| ua.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );
        debug_info.insert("Last-Time".to_string(), Utc::now().to_rfc3339());
        debug_info.insert("Cf-Connecting-Ip".to_string(),
            res.request().headers().get("Cf-Connecting-Ip")
                .and_then(|ip| ip.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );
        debug_info.insert("Accept-Encoding".to_string(),
            res.request().headers().get("Accept-Encoding")
                .and_then(|ae| ae.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );
        debug_info.insert("Accept-Language".to_string(),
            res.request().headers().get("Accept-Language")
                .and_then(|al| al.to_str().ok())
                .unwrap_or("Unknown").to_string()
        );

        // Teraコンテキストを作成
        let mut context = Context::new();
        context.insert("code", &status_code.to_string());
        context.insert("ms", &status_message);
        context.insert("color", &status_color);
        context.insert("suggestions", &suggestion_list);
        context.insert("debug_info", &debug_info);

        // テンプレートをレンダリング
        let rendered = self.err_page_template.render("err_template.html", &context)
            .unwrap_or_else(|err| {
                eprintln!("Template rendering error: {}", err);
                "Error rendering template".to_string()
            });

        HttpResponse::build(res.status())
            .content_type("text/html")
            .body(rendered)
    }
}