#[macro_use]
extern crate log;

mod about_page;
mod app_error;
mod compliance;
mod config;
mod host;
mod html_page;
mod illustrations;
mod live_poll;
mod live_poll_store;
mod play;
mod session_id;
mod slide;
mod static_file;
mod svg_icons;
mod wsmessage;

use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use app_error::AppError;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    env_logger::init();
    static_file::init();

    runtime.block_on(async {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        info!("Listening on http://{}", addr);

        let routes = axum::Router::new()
            .route("/", get(host::get_poll_page))
            .route("/p", get(play::get_play_page))
            .route("/start_poll", post(host::post_start_poll))
            .route("/stop_poll/:poll_id", post(host::post_stop_poll))
            .route("/ws/host/:poll_id", get(host::host_socket))
            .route("/ws/p/:poll_id", get(play::play_socket))
            //.route("/about", get(about_page::get_about_page))
            /*.route("/submit_mc_answer/:poll_id", post(play::post_mc_answer))
            .route(
                "/submit_free_text_answer/:poll_id",
                post(play::post_free_text_answer),
            )*/
            //.route("/name_avatar/:poll_id", post(play::post_name_avatar))
            .route("/static/:file_name", get(static_file::http_get_static_file))
            .route("/data-privacy", get(compliance::get_privacy_policy_page))
            .route(
                "/terms-of-service",
                get(compliance::get_terms_of_service_page),
            )
            .route("/cookie-policy", get(compliance::get_cookie_policy_page))
            .route("/contact", get(compliance::get_contact_page))
            .route("/robots.txt", get(compliance::get_robots_txt))
            .fallback(get(get_fallback));

        axum::serve(listener, routes).await.unwrap();
    })
}

async fn get_fallback() -> Response {
    AppError::NotFound.into_response()
}
