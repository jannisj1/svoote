use axum::response::{IntoResponse, Response};
use maud::html;

use crate::{app_error::AppError, html_page};

pub async fn get_privacy_policy_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page(
        "Privacy policy - Svoote",
        html! {
            div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/privacy/e569e7fe-436d-11ef-9615-cb709ba43f2f"
            {}
        },
        true,
    )
    .into_response());
}

pub async fn get_terms_of_service_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page(
        "Terms of service - Svoote",
        html! {
            div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/tos/e569e7fe-436d-11ef-9615-cb709ba43f2f"
            {}
        },
        true,
    )
    .into_response());
}

pub async fn get_cookie_policy_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page(
        "Cookie policy - Svoote",
        html! {
            div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/cookies/e569e7fe-436d-11ef-9615-cb709ba43f2f"
            {}
        },
        true,
    )
    .into_response());
}

pub async fn get_contact_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page(
        "Contact - Svoote",
        html! {
            ."text-slate-700" {
                ."my-8 text-2xl font-semibold" { "Svoote - Contact" }
                ."mb-2" { "Svoote.com is owned and operated by" }
                ."mb-4" { "Jannis Jelten" br; "Zimmermannstrasse 16b" br; "37075 Göttingen" br; "Germany" }
                ."" { "Contact us at:" br; "info@svoote.com" }
            }
        }, true
    )
    .into_response());
}

pub async fn get_robots_txt() -> Response {
    return r#"
User-agent: *
Disallow: /p
        "#
    .into_response();
}
