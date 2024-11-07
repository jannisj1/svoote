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
mod polls;
mod session_id;
mod slide;
mod static_file;
mod svg_icons;
mod word_cloud;

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
            .route("/about", get(about_page::get_about_page))
            .route("/about/demo_mc", get(about_page::get_mc_start_page_demo))
            .route("/about/demo_ft", get(about_page::get_ft_start_page_demo))
            .route("/", get(polls::get_poll_page).post(polls::post_start_poll))
            .route("/next_slide/:poll_id", post(host::post_next_slide))
            .route("/previous_slide/:poll_id", post(host::post_previous_slide))
            .route("/exit_poll/:poll_id", post(host::post_exit_poll))
            .route("/sse/host_question/:poll_id", get(host::get_sse_slides))
            .route("/sse/host_results/:poll_id", get(host::get_sse_statistics))
            .route("/sse/leaderboard/:poll_id", get(host::get_sse_leaderboard))
            .route(
                "/sse/participant_counter/:poll_id",
                get(host::get_sse_user_counter),
            )
            .route("/p", get(play::get_play_page))
            .route("/submit_mc_answer/:poll_id", post(play::post_mc_answer))
            .route(
                "/submit_free_text_answer/:poll_id",
                post(play::post_free_text_answer),
            )
            .route("/sse/play/:quiz_id", get(play::get_sse_play))
            .route("/name_avatar/:poll_id", post(play::post_name_avatar))
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
