use tera::Tera;

#[derive(Clone)]
pub struct AppConfig {
    pub server_bind: String,
    pub server_backlog: u32,
    pub server_workers: usize,
    pub template: Tera,
}

impl AppConfig {
    pub fn new() -> Self {
        
        let app_config = AppConfig {
            server_bind: "0.0.0.0:83".to_string(),
            server_backlog: 512,
            server_workers: 16,
            template: match Tera::new("templates/**/*") {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Parsing error(s): {}", e);
                    std::process::exit(1);
                }
            },
        };
        app_config
    }
}