#[macro_use]
extern crate log;

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");

mod app_error;
mod compliance;
mod config;
mod host;
mod html_page;
//mod illustrations;
mod live_poll;
mod live_poll_store;
mod play;
mod session_id;
mod slide;
mod start_page;
mod static_file;
mod svg_icons;
mod wsmessage;

use accept_language::intersection;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use app_error::AppError;
use axum_extra::extract::CookieJar;
use smartstring::{Compact, SmartString};

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    env_logger::init();
    static_file::init();

    if let Err(e) = dotenv::dotenv() {
        error!("Error parsing .env-file: {}", e);
    }

    runtime.block_on(async {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        info!("Listening on http://{}", addr);

        let routes = axum::Router::new()
            .route("/", get(start_page::get_start_page))
            .route("/host", get(host::get_host_page))
            .route("/p", get(play::get_play_page))
            .route("/poll_exists/:poll_id", get(play::get_poll_exists))
            .route("/start_poll", post(host::post_start_poll))
            .route("/stop_poll/:poll_id", post(host::post_stop_poll))
            .route("/ws/host/:poll_id", get(host::host_socket))
            .route("/ws/p/:poll_id", get(play::play_socket))
            .route("/submit_mc_answer/:poll_id", post(play::post_mc_answer))
            .route("/submit_ft_answer/:poll_id", post(play::post_ft_answer))
            //.route("/name_avatar/:poll_id", post(play::post_name_avatar))
            .route("/static/:file_name", get(static_file::http_get_static_file))
            .route("/data-privacy", get(compliance::get_privacy_policy_page))
            .route(
                "/terms-of-service",
                get(compliance::get_terms_of_service_page),
            )
            .route("/cookie-policy", get(compliance::get_cookie_policy_page))
            .route("/manage-cookies", get(compliance::get_manage_cookies_page))
            .route("/contact", get(compliance::get_contact_page))
            .route("/robots.txt", get(compliance::get_robots_txt))
            .route("/bombardft/:poll_id", get(host::get_bombardft))
            .route("/stats", get(host::get_stats))
            .fallback(get(get_fallback));

        axum::serve(listener, routes).await.unwrap();
    })
}

async fn get_fallback() -> Response {
    return AppError::NotFound.into_response();
}

pub fn select_language(cookies: &CookieJar, headers: &HeaderMap) -> SmartString<Compact> {
    let languages = ["en", "de"];
    if let Some(lang_cookie) = cookies.get("lang") {
        let lang_cookie = lang_cookie.value_trimmed();
        for lang in languages {
            if lang_cookie == lang {
                return SmartString::from(lang);
            }
        }
    }
    if let Some(lang_header) = headers.get(ACCEPT_LANGUAGE) {
        if let Ok(lang_header) = lang_header.to_str() {
            if let Some(lang) = intersection(lang_header, &["en", "de"]).get(0) {
                return SmartString::from(lang);
            }
        }
    }

    return SmartString::from("en");
}
