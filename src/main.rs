use actix_web::dev::ServiceResponse;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::{ErrorHandlerResponse, Logger};
use env_logger::Env;
use sys::app_set;
use tera::Context;

use crate::sys::app_set::AppSet;
use crate::sys::init::AppConfig;

mod sys;
mod handler;

#[actix_web::get("/")]
async fn index(app_set: web::Data<AppSet>, req: HttpRequest) -> impl Responder {
    let mut context = Context::new();
    context.insert("color", "#ffffff");

    let response = app_set.app_config.template.render("index.html", &context).unwrap_or_else(|err| {
        eprint!("Template Error: {:?}", err);
        "Err...".to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}

#[actix_web::get("/license")]
async fn license(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    let mut context = Context::new();
    context.insert("color", "#777777");

    let response = app_set.app_config.template.render("license.html", &context).unwrap_or_else(|err| {
        eprint!("Template Error: {:?}", err);
        "Err...".to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}

#[actix_web::get("/terms")]
async fn terms(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    let mut context = Context::new();
    context.insert("color", "#777777");

    let response = app_set.app_config.template.render("terms.html", &context).unwrap_or_else(|err| {
        eprint!("Template Error: {:?}", err);
        "Err...".to_string()
    });

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}

#[actix_web::get("/371tti_icon.jpg")]
async fn tti_icon(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    if let Some(file_data) = app_set.static_cache.get("371tti_icon.jpg") {
        HttpResponse::Ok()
            .content_type("image/jpg")
            .body(file_data.clone())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/favicon.ico")]
async fn icon(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    if let Some(file_data) = app_set.static_cache.get("favicon.ico") {
        HttpResponse::Ok()
            .content_type("image/x-icon")
            .body(file_data.clone())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/robots.txt")]
async fn robots(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    if let Some(file_data) = app_set.static_cache.get("robots.txt") {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(file_data.clone())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/ads.txt")]
async fn ads(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    if let Some(file_data) = app_set.static_cache.get("ads.txt") {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(file_data.clone())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/banner.jpg")]
async fn banner(app_set: web::Data<AppSet>, _req: HttpRequest) -> impl Responder {
    if let Some(file_data) = app_set.static_cache.get("banner.jpg") {
        HttpResponse::Ok()
            .content_type("image/jpg")
            .body(file_data.clone())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/err/{statuscode}")]
pub async fn error_test(statuscode: web::Path<u16>) -> impl Responder {
    let status_code = *statuscode;
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code)
        .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR))
        .finish()
}


fn err_handler<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let app_set = res.request().app_data::<web::Data<AppSet>>().unwrap();
    let response = app_set.err_handler.page_generate(&res);
    return Ok(ErrorHandlerResponse::Response(res.into_response(response.map_into_right_body())));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_config = AppConfig::new();

    let app_set_instance = AppSet::new(app_config.clone()).await;

    let app_set = web::Data::new(app_set_instance);
    
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::ErrorHandlers::new().default_handler(err_handler))
            .app_data(app_set.clone())
            .service(index)
            .service(error_test)
            .service(tti_icon)
            .service(banner)
            .service(icon)
            .service(robots)
            .service(ads)
            .service(license)
            .service(terms)
    })
    .bind(app_config.server_bind.clone())?
    .workers(app_config.server_workers.clone())
    .backlog(app_config.server_backlog.clone())
    .run();
    
    server.await?;

    Ok(())
}