use axum::response::{IntoResponse, Response};
use maud::html;

use crate::html_page;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    DatabaseError(tower_sessions::session::Error),
    OtherInternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => {
                return (
                    axum::http::StatusCode::NOT_FOUND,
                    html_page::render_html_page("Svoote - 404 Not Found", html! {
                        h1 ."mt-20 text-slate-900 font-extrabold text-3xl sm:text-4xl lg:text-5xl tracking-tight text-center" { "404 - Not found" }
                        p ."mt-4 mb-20 text-lg text-slate-600 text-center" { "Unfortunately, the webpage at this address does not exist (anymore)." }
                    }, true).into_response()
                ).into_response();
            }
            AppError::BadRequest(msg) => {
                return (axum::http::StatusCode::BAD_REQUEST, msg).into_response();
            }
            AppError::DatabaseError(e) => {
                error!("Redis database error: {e}");
                return generate_500_response();
            }
            AppError::OtherInternalServerError(s) => {
                error!("Other internal server error: {s}");
                return generate_500_response();
            }
        }
    }
}

impl core::fmt::Display for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Self::NotFound => write!(f, "Not Found")?,
            Self::BadRequest(s) => write!(f, "Bad request: {s}")?,
            Self::DatabaseError(e) => write!(f, "Database error: {e}")?,
            Self::OtherInternalServerError(s) => write!(f, "Internal server error: {s}")?,
        }

        Ok(())
    }
}

fn generate_500_response() -> Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        html_page::render_html_page(
            "Svoote - 500 Internal server error",
            html! {
                h1 ."mt-20 text-slate-900 font-extrabold text-3xl sm:text-4xl lg:text-5xl tracking-tight text-center" { "500 - Internal server error" }
                p ."mt-4 mb-20 text-lg text-slate-600 text-center" { "Something went wrong, we are working on fixing the issue." }
            },true
        )
        .into_response(),
    )
        .into_response()
}
