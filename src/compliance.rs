use std::env;

use axum::{
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use maud::html;

use crate::{
    app_error::AppError,
    html_page::{self, render_header},
    select_language,
    svg_icons::SvgIcon,
};

pub async fn get_privacy_policy_page(
    cookies: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    return Ok(html_page::render_html_page(
        "Privacy policy - Svoote",
        &l,
        html! {
            (render_header(html!{}))
            ."mx-6 sm:mx-14 my-24 text-slate-500" {
                ."max-w-2xl mx-auto" {
                    h1 ."mb-2 text-slate-700 text-4xl font-bold" { (t!("privacy_policy_title", locale=l)) }
                    h3 ."mb-6 text-sm" { (t!("privacy_last_updated", locale=l)) }
                    p ."mb-8" { (t!("privacy_policy_tldr", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("what_data_we_collect", locale=l)) }
                    p ."mb-4" { (t!("what_data_we_collect_p", locale=l)) }
                    p ."mb-8" {
                        (t!("what_data_we_collect_p2", locale=l))
                        a ."underline" href="https://plausible.io/data-policy" { "Plausible data policy" } "."
                    }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("your_rights", locale=l)) }
                    p ."mb-8" { (t!("your_rights_p", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("changes_and_questions", locale=l)) }
                    p ."mb-4" { (t!("changes_and_questions_p", locale=l)) }
                    p ."mb-8" { (t!("changes_and_questions_p2", locale=l)) }
                }
            }
            /*div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/privacy/e569e7fe-436d-11ef-9615-cb709ba43f2f"
            {}*/
        },
        true
    )
    .into_response());
}

pub async fn get_terms_of_service_page(
    cookies: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    return Ok(html_page::render_html_page(
        "Terms of service - Svoote",
        &l,
        html! {
            (render_header(html!{}))
            ."mx-6 sm:mx-14 my-24 text-slate-500" {
                ."max-w-2xl mx-auto" {

                    h1 ."mb-2 text-slate-700 text-4xl font-bold" { (t!("tos_title", locale=l)) }
                    h3 ."mb-6 text-sm" { (t!("tos_last_updated", locale=l)) }
                    p ."mb-8" { (t!("tos_prelim", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_1_title", locale=l)) }
                    p ."mb-4" { (t!("tos_1_0", locale=l)) }
                    p ."mb-4" { (t!("tos_1_1", locale=l)) }
                    p ."mb-8" { (t!("tos_1_2", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_2_title", locale=l)) }
                    p ."mb-4" { (t!("tos_2_0", locale=l)) }
                    p ."mb-4" { (t!("tos_2_1", locale=l)) }
                    p ."mb-8" { (t!("tos_2_2", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_3_title", locale=l)) }
                    p ."mb-4" { (t!("tos_3_0", locale=l)) }
                    p ."mb-8" { (t!("tos_3_1", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_4_title", locale=l)) }
                    p ."mb-4" { (t!("tos_4_0", locale=l)) }
                    p ."mb-8" { (t!("tos_4_1", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_5_title", locale=l)) }
                    p ."mb-4" { (t!("tos_5_0", locale=l)) }
                    p ."mb-4" { (t!("tos_5_1", locale=l)) }
                    p ."mb-8" { (t!("tos_5_2", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_6_title", locale=l)) }
                    p ."mb-8" { (t!("tos_6_0", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_7_title", locale=l)) }
                    p ."mb-4" { (t!("tos_7_0", locale=l)) }
                    p ."mb-8" { (t!("tos_7_1", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("tos_8_title", locale=l)) }
                    p ."mb-8" { (t!("tos_8_0", locale=l)) }
                }
            }
            /*div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/tos/e569e7fe-436d-11ef-9615-cb709ba43f2f"
                {}*/
        },
        true
    )
    .into_response());
}

pub async fn get_cookie_policy_page(
    cookies: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    return Ok(html_page::render_html_page(
        "Cookie policy - Svoote",
        &l,
        html! {
            (render_header(html!{}))
            ."mx-6 sm:mx-14 my-24 text-slate-500" {
                ."max-w-2xl mx-auto" {
                    h1 ."mb-2 text-slate-700 text-4xl font-bold" { (t!("cookies_title", locale=l)) }
                    h3 ."mb-6 text-sm" { (t!("cookies_last_updated", locale=l)) }
                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("what_are_cookies", locale=l)) }
                    p ."mb-6" { (t!("what_are_cookies_p", locale=l)) }

                    h2 ."mb-4 text-slate-700 text-2xl font-semibold" { (t!("how_we_use_cookies", locale=l)) }
                    p ."mb-4" { (t!("how_we_use_cookies_p", locale=l)) }
                    ul ."mb-4 ml-2 list-disc list-inside" {
                        li ."mb-2" { (t!("strictly_necessary_cookies", locale=l)) }
                        li ."mb-2" { (t!("preference_cookies", locale=l)) }
                    }
                }
            }
            /*div #"__enzuzo-root" {}
            script
                #"__enzuzo-root-script"
                src="https://app.enzuzo.com/scripts/cookies/e569e7fe-436d-11ef-9615-cb709ba43f2f"
            {}*/
        },
        true
    )
    .into_response());
}

pub async fn get_contact_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page(
        "Contact - Svoote",
        "en",
        html! {
            (render_header(html!{}))
            ."mx-6 sm:mx-14 my-32 text-center text-slate-700" {
                ."mb-2" { "This website (" (env::var("DOMAIN_NAME").unwrap_or("DOMAIN_NAME missing".to_string())) ") is owned and operated by" }
                ."mb-4" {
                    (env::var("CONTACT_1").unwrap_or("CONTACT_1 missing".to_string())) br;
                    (env::var("CONTACT_2").unwrap_or("CONTACT_2 missing".to_string())) br;
                    (env::var("CONTACT_3").unwrap_or("CONTACT_3 missing".to_string())) br;
                    (env::var("CONTACT_4").unwrap_or("CONTACT_4 missing".to_string()))
                }
                ."" { "Contact us at:" br; (env::var("CONTACT_EMAIL").unwrap_or("CONTACT_EMAIL missing".to_string())) }
            }
        },
        true
    )
    .into_response());
}

pub async fn get_manage_cookies_page(
    cookies: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    return Ok(html_page::render_html_page(
        "Manage cookies - Svoote",
        &l,
        html! {
            (render_header(html!{}))
                div ."mx-6 sm:mx-14 my-32" {
                    div ."mx-auto max-w-lg" {
                        h1 ."mb-3 flex items-center gap-2 text-slate-700 text-xl font-medium"
                            { div ."size-5" { (SvgIcon::Cookie.render()) } "Cookies" }
                        p ."mb-4 text-slate-500 text-sm" {
                            (t!("cookie_banner_text", locale=l)) a href="/cookie-policy" ."underline" { "Cookie Policy"} "."
                        }
                        div class="mb-6 sm:mb-0 flex gap-2" {
                            input type="checkbox" id="disabled-switch" class="peer hidden" disabled {}
                            label for="disabled-switch"
                                class="w-10 h-6 flex items-center bg-gray-300 rounded-full p-1"
                                { div class="w-4 h-4 bg-gray-500 rounded-full shadow-md translate-x-4" {} }
                                span class="text-gray-500" { (t!("necessary_cookies", locale=l)) }
                        }
                        div ."flex flex-wrap sm:justify-end gap-4" {
                            button onclick="localStorage.setItem('cookiesAccepted', 'true'); window.location.href = '/';"
                                ."w-full sm:w-auto px-5 py-2 bg-cyan-700 text-white text-sm font-semibold shadow-xl cursor-pointer hover:bg-cyan-600"
                                { (t!("cookie_banner_accept", locale=l)) }
                        }
                    }
                }
        },
        true
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
