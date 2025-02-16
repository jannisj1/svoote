use crate::{
    app_error::AppError,
    html_page::{self, render_header, render_start_page_menu_bar},
    select_language,
    svg_icons::SvgIcon,
};
use axum::{
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use maud::{html, Markup};

pub async fn get_start_page(cookies: CookieJar, headers: HeaderMap) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);

    return Ok(html_page::render_html_page("Svoote", &l, html! {
        (render_header(render_start_page_menu_bar(&l)))
        (render_join_form(&l))
        div ."mt-16 mx-6 sm:mx-14" {
            div ."max-w-2xl mx-auto" {
                h1 ."mx-auto max-w-xl mb-3 text-center text-slate-800 text-5xl font-bold leading-tight" { (t!("title", locale=l)) }
                h2 ."mb-8 text-sm text-center text-slate-500 leading-8" {
                    (t!("subtitle", locale=l)) " "
                    a href="/#pricing" ."underline text-cyan-600" { (t!("pricing_and_limits", locale=l)) " ↗" }
                }
                div ."mb-12 px-2 py-1.5 bg-slate-700 rounded-lg" {
                    div ."mb-2 flex justify-between gap-1" {
                        div ."flex-1 mt-0.5 ml-1 text-xs text-white font-semibold" { "Svoote" div ."inline-block ml-1 size-2.5 translate-y-[1px]" { (SvgIcon::Rss.render()) } }
                        div ."size-2 rounded-full bg-green-500" {}
                        div ."size-2 rounded-full bg-orange-400" {}
                        div ."size-2 rounded-full bg-rose-500" {}
                    }
                    video autoplay loop muted playsinline ."appearance-none rounded-xs" {
                        source src="/img/svoote_demo.webm" type="video/webm";
                        "Your browser does not support video playback."
                    }
                }
                h2 ."mb-4 text-center text-slate-600 font-bold" { (t!("what_do_you_ask", locale=l)) }
                div ."mb-3 flex justify-center" {
                    a ."px-6 py-2 text-white text-lg font-semibold bg-cyan-600 rounded-full hover:bg-cyan-700"
                        href="/host"
                        { (t!("create_presentation_action_btn", locale=l)) }
                }
                h3 ."mb-40 text-xs text-center text-slate-500" {
                    (t!("sub_action_btn_1", locale=l)) br;
                    (t!("sub_action_btn_2", locale=l))
                }
            }
            h3 ."mb-10 text-center text-slate-700 text-4xl font-bold" id="why" { (t!("why_svoote", locale=l)) }
            section ."mb-32 grid md:grid-cols-2 gap-10 text-slate-700" {
                div ."flex flex-col gap-10" {
                    div ."px-6 py-5 bg-green-100 rounded-lg" {
                        h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                            { ."size-5" { (SvgIcon::ShoppingCart.render()) } (t!("low_pricing_section_title", locale=l)) }
                        p ."" {
                            (t!("low_pricing_section_text", locale=l))
                        }
                    }
                    div ."px-6 py-5 bg-orange-100 rounded-lg" {
                        h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                            { ."size-5" { (SvgIcon::Image.render()) } (t!("ad_free_section_title", locale=l)) }
                        p ."" {
                            (t!("ad_free_section_text", locale=l))
                        }
                    }
                }
                div ."flex flex-col gap-10" {
                    ."px-6 py-5 bg-rose-100 rounded-lg" {
                        h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                            { ."size-5" { (SvgIcon::Github.render()) } (t!("open_source_section_title", locale=l)) }
                        p ."" {
                            (t!("open_source_section_text", locale=l))
                            a ."underline" href="https://github.com/jannisj1/svoote" { "Github" } "."
                        }
                    }
                    div ."px-6 py-5 bg-cyan-100 rounded-lg" {
                        h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                            { ."size-5" { (SvgIcon::Lock.render()) } (t!("privacy_section_title", locale=l)) }
                        p ."" {
                            (t!("privacy_section_text_1", locale=l))
                            a ."underline" href="https://plausible.io" { "Plausible" }
                            (t!("privacy_section_text_2", locale=l))
                        }
                    }
                }
            }
            h3 ."mb-10 text-center text-slate-700 text-4xl font-bold" id="pricing" { (t!("pricing_and_limits", locale=l)) }
            section ."mb-32 flex justify-center gap-10 sm:gap-20 flex-wrap" {
                div ."w-64 p-8 bg-white rounded-lg border shadow-xs" {
                    h1 ."mb-4 text-2xl text-slate-900 font-medium tracking-tight" { "Free" }
                    ."mb-6 flex justify-start items-baseline gap-2"
                        { ."text-4xl text-slate-900" { "$0" } ."text-slate-500" { (t!("usd_per_month", locale=l)) } }
                    a ."mb-16 block w-fit px-5 py-3 bg-slate-100 text-slate-800 font-medium rounded-xl hover:bg-slate-200 transition"
                        href="/host" { (t!("start_now", locale=l)) }
                    div ."mb-2 text-sm text-slate-800 tracking-wide"
                        { (t!("whats_included_in", locale=l)) span ."font-medium tracking-tight" { "Free" } ":" }
                    ul ."flex flex-col gap-1 text-sm text-slate-800" {
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("unlimited_polls", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("up_to_100_users", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("multiple_choice_slides", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("word_cloud_slides", locale=l)) }
                    }
                }
                div ."w-64 p-8 bg-white rounded-lg border shadow-xs" {
                    h1 ."mb-4 text-2xl text-slate-900 font-medium tracking-tight" { "Pro" }
                    ."mb-6 flex justify-start items-baseline gap-2"
                        { ."text-4xl text-slate-900" { "$4" } ."text-slate-500" { (t!("usd_per_month", locale=l)) } }
                    ."mb-16 block w-fit px-5 py-3 bg-slate-800 text-slate-100 font-medium rounded-xl"
                        { (t!("not_available_yet", locale=l)) }
                    div ."mb-2 text-sm text-slate-800 tracking-wide"
                        { (t!("whats_included_in", locale=l)) span ."font-medium tracking-tight" { "Pro" } ":" }
                    ul ."flex flex-col gap-1 text-sm text-slate-800" {
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("everything_in_free", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("unlimited_users", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("quiz_competition", locale=l)) }
                        li ."flex items-center gap-2" { ."size-3.5 shrink-0" { (SvgIcon::Check.render()) } (t!("more_features_future", locale=l)) }
                    }
                }
            }
        }
    }).into_response());
}

pub fn render_join_form(l: &str) -> Markup {
    return html! {
        div ."mx-6 sm:mx-14" {
            form onsubmit="event.preventDefault(); joinPoll(); return false;"
                ."mx-auto w-fit px-6 py-4 flex flex-wrap justify-center items-center gap-x-4 gap-y-3 text-base bg-cyan-100 rounded-xl"
            {
                label ."text-slate-600 font-medium" for="poll-id-input"
                    { (t!("enter_poll_desc", locale=l)) }
                div."flex items-center gap-1 text-slate-600 font-semibold" {
                    "#" input id="poll-id-input" name="c" type="text" pattern="[0-9]*" inputmode="numeric" placeholder="1234"
                    ."w-20 px-3 py-1 border-2 border-slate-400 rounded-lg outline-hidden";
                    button ."ml-3 px-6 py-1.5 text-white font-semibold bg-slate-600 rounded-full cursor-pointer hover:bg-slate-500"
                        { (t!("join_btn_desc", locale=l)) }
                }
            }
        }
    };
}
