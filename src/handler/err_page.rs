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
        let mut status_color = HashMap::new();
        status_color.insert(4, "#ff9900ff".to_string());
        status_color.insert(5, "#ff0000bb".to_string());

        let mut status_message = HashMap::new();
        status_message.insert(400, "BadRequest".to_string());
        status_message.insert(401, "Unauthorized".to_string());
        status_message.insert(403, "Forbidden".to_string());
        status_message.insert(404, "NotFound".to_string());
        status_message.insert(405, "MethodNotAllowed".to_string());
        status_message.insert(406, "NotAcceptable".to_string());
        status_message.insert(408, "RequestTimeout".to_string());
        status_message.insert(409, "Conflict".to_string());
        status_message.insert(410, "Gone".to_string());
        status_message.insert(411, "LengthRequired".to_string());
        status_message.insert(412, "PreconditionFailed".to_string());
        status_message.insert(413, "RequestEntityTooLarge".to_string());
        status_message.insert(414, "RequestURITooLarge".to_string());
        status_message.insert(415, "UnsupportedMediaType".to_string());
        status_message.insert(416, "RequestedRangeNotSatisfiable".to_string());
        status_message.insert(417, "ExpectationFailed".to_string());
        status_message.insert(418, "ImATeapot".to_string());
        status_message.insert(422, "UnprocessableEntity".to_string());
        status_message.insert(423, "Locked".to_string());
        status_message.insert(424, "FailedDependency".to_string());
        status_message.insert(428, "PreconditionRequired".to_string());
        status_message.insert(429, "TooManyRequests".to_string());
        status_message.insert(431, "RequestHeaderFieldsTooLarge".to_string());
        status_message.insert(451, "UnavailableForLegalReasons".to_string());
        status_message.insert(500, "InternalServerError".to_string());
        status_message.insert(501, "NotImplemented".to_string());
        status_message.insert(502, "BadGateway".to_string());
        status_message.insert(503, "ServiceUnavailable".to_string());
        status_message.insert(504, "GatewayTimeout".to_string());
        status_message.insert(505, "HTTPVersionNotSupported".to_string());
        status_message.insert(506, "VariantAlsoNegotiates".to_string());
        status_message.insert(507, "InsufficientStorage".to_string());
        status_message.insert(508, "LoopDetected".to_string());
        status_message.insert(510, "NotExtended".to_string());
        status_message.insert(511, "NetworkAuthenticationRequired".to_string());

        let mut suggestion_fix_message = HashMap::new();

        // 4xx クライアントエラー
        suggestion_fix_message.insert(400, {
            let mut map = HashMap::new();
            map.insert(1, "Check the request syntax".to_string());
            map.insert(2, "Verify the request parameters".to_string());
            map.insert(3, "Ensure the URL is correct".to_string());
            map
        });
        suggestion_fix_message.insert(401, {
            let mut map = HashMap::new();
            map.insert(1, "Check the authentication credentials".to_string());
            map.insert(2, "Login again".to_string());
            map.insert(3, "Contact the website administrator".to_string());
            map
        });
        suggestion_fix_message.insert(403, {
            let mut map = HashMap::new();
            map.insert(1, "Check the URL for errors".to_string());
            map.insert(2, "Request access from the administrator".to_string());
            map.insert(3, "Ensure you have the necessary permissions".to_string());
            map
        });
        suggestion_fix_message.insert(404, {
            let mut map = HashMap::new();
            map.insert(1, "Check the URL".to_string());
            map.insert(2, "Reload the page".to_string());
            map.insert(3, "Clear the browser cache".to_string());
            map.insert(4, "Try using another browser".to_string());
            map.insert(5, "Contact customer support".to_string());
            map
        });
        suggestion_fix_message.insert(405, {
            let mut map = HashMap::new();
            map.insert(1, "Check the request method (GET, POST, etc.)".to_string());
            map.insert(2, "Refer to the website's API documentation".to_string());
            map.insert(3, "Ensure the method is supported".to_string());
            map
        });
        suggestion_fix_message.insert(406, {
            let mut map = HashMap::new();
            map.insert(1, "Check the requested media type.".to_string());
            map.insert(2, "Ensure server supports the requested format.".to_string());
            map
        });
        suggestion_fix_message.insert(407, {
            let mut map = HashMap::new();
            map.insert(1, "Verify proxy authentication.".to_string());
            map.insert(2, "Contact network administrator for proxy details.".to_string());
            map
        });
        suggestion_fix_message.insert(408, {
            let mut map = HashMap::new();
            map.insert(1, "Check your internet connection".to_string());
            map.insert(2, "Ensure the server is not overloaded".to_string());
            map.insert(3, "Retry the request after a moment".to_string());
            map
        });
        suggestion_fix_message.insert(409, {
            let mut map = HashMap::new();
            map.insert(1, "Resolve conflicting resources.".to_string());
            map.insert(2, "Ensure request data is consistent.".to_string());
            map
        });
        suggestion_fix_message.insert(410, {
            let mut map = HashMap::new();
            map.insert(1, "This resource is no longer available.".to_string());
            map.insert(2, "Contact the website administrator for information.".to_string());
            map
        });
        suggestion_fix_message.insert(411, {
            let mut map = HashMap::new();
            map.insert(1, "Set 'Content-Length' header in request.".to_string());
            map
        });
        suggestion_fix_message.insert(412, {
            let mut map = HashMap::new();
            map.insert(1, "Verify request preconditions.".to_string());
            map.insert(2, "Adjust precondition headers.".to_string());
            map
        });
        suggestion_fix_message.insert(413, {
            let mut map = HashMap::new();
            map.insert(1, "Reduce the request entity size.".to_string());
            map.insert(2, "Contact administrator for size limits.".to_string());
            map
        });
        suggestion_fix_message.insert(414, {
            let mut map = HashMap::new();
            map.insert(1, "Simplify the URL length.".to_string());
            map.insert(2, "Use a shorter URL structure.".to_string());
            map
        });
        suggestion_fix_message.insert(415, {
            let mut map = HashMap::new();
            map.insert(1, "Check the media type in request.".to_string());
            map.insert(2, "Ensure server supports media type.".to_string());
            map
        });
        suggestion_fix_message.insert(416, {
            let mut map = HashMap::new();
            map.insert(1, "Check requested range headers.".to_string());
            map
        });
        suggestion_fix_message.insert(417, {
            let mut map = HashMap::new();
            map.insert(1, "Check 'Expect' request header.".to_string());
            map
        });
        suggestion_fix_message.insert(418, {
            let mut map = HashMap::new();
            map.insert(1, "I'm a teapot, not a coffee machine.".to_string());
            map
        });
        suggestion_fix_message.insert(422, {
            let mut map = HashMap::new();
            map.insert(1, "Check request syntax and data.".to_string());
            map
        });
        suggestion_fix_message.insert(429, {
            let mut map = HashMap::new();
            map.insert(1, "Reduce the frequency of requests.".to_string());
            map.insert(2, "Wait before sending more requests.".to_string());
            map
        });
        suggestion_fix_message.insert(431, {
            let mut map = HashMap::new();
            map.insert(1, "Reduce header data size.".to_string());
            map
        });
        
        // 5xx サーバーエラー
        suggestion_fix_message.insert(500, {
            let mut map = HashMap::new();
            map.insert(1, "Wait a few moments and retry the request".to_string());
            map.insert(2, "Check the website's social media for updates".to_string());
            map.insert(3, "Contact customer support".to_string());
            map
        });
        suggestion_fix_message.insert(501, {
            let mut map = HashMap::new();
            map.insert(1, "Verify the request method is correct".to_string());
            map.insert(2, "Check if the feature is implemented".to_string());
            map.insert(3, "Contact the website administrator".to_string());
            map
        });
        suggestion_fix_message.insert(502, {
            let mut map = HashMap::new();
            map.insert(1, "Check your internet connection".to_string());
            map.insert(2, "Wait a few moments and retry the request".to_string());
            map.insert(3, "Contact the website if the issue persists".to_string());
            map
        });
        suggestion_fix_message.insert(503, {
            let mut map = HashMap::new();
            map.insert(1, "Check if the website is under maintenance".to_string());
            map.insert(2, "Wait and retry later".to_string());
            map.insert(3, "Contact the website for more information".to_string());
            map
        });
        suggestion_fix_message.insert(504, {
            let mut map = HashMap::new();
            map.insert(1, "Check your internet connection".to_string());
            map.insert(2, "Ensure the server is reachable".to_string());
            map.insert(3, "Retry the request after a moment".to_string());
            map
        });
        suggestion_fix_message.insert(505, {
            let mut map = HashMap::new();
            map.insert(1, "Verify the HTTP version used.".to_string());
            map.insert(2, "Contact administrator to check version support.".to_string());
            map
        });
        suggestion_fix_message.insert(511, {
            let mut map = HashMap::new();
            map.insert(1, "Authenticate to access network.".to_string());
            map
        });
        



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

        let is_inline_request = res.request()
            .headers()
            .get("Content-Disposition")
            .map_or(false, |header| header.to_str().map_or(false, |value| value.contains("inline")));

        if is_inline_request {
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
        } else {
            // APIなどのブラウザ表示でないリクエストの場合
            HttpResponse::build(res.status())
                .content_type("text/plain")
                .body(status_message)
        }
    }
}