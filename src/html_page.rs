use maud::{html, Markup, DOCTYPE};

use crate::{config::COLOR_PALETTE, static_file, svg_icons::SvgIcon};

pub fn render_html_page(title: &str, l: &str, main_content: maud::Markup) -> maud::Markup {
    html! {
        (DOCTYPE)
        html lang=(l) {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta http-equiv="X-UA-Compatible" content="ie=edge";
                meta name="description" content=(t!("html_description_meta", locale=l));
                meta name="google-site-verification" content="43Qm55o4cXLfJSwfb_7gvXUUFYiYzs7zvKqUX46pk1c";
                title { (title) }
                @if cfg!(debug_assertions) { script src="https://unpkg.com/@tailwindcss/browser@4" {} }
                link rel="stylesheet" href=(static_file::get_path("bundle.css"));
                script defer src=(static_file::get_path("app.js")) {}
                link rel="icon" type="image/png" href="/img/svoote_icon_t.png";
                @if let Ok(domain) = std::env::var("PLAUSIBLE_DOMAIN") {
                    @if !cfg!(debug_assertions) { script defer data-domain=(domain) src="https://plausible.io/js/script.js" {} }
                }
                script {
                    "let colorPalette = [" @for color in COLOR_PALETTE { "'" (color) "'," } "];"
                }
            }
            body ."group min-h-screen flex flex-col text-slate-700" {
                main ."flex-1 mx-auto w-full max-w-[1408px]" {
                    (main_content)
                }
                footer ."mt-4 px-4 py-8 text-xs text-slate-500 bg-slate-100" {
                    div ."mb-6 flex justify-center items-center flex-wrap gap-4" {
                        a href="/" ."flex items-baseline gap-1.5 text-cyan-600" {
                            span ."text-lg tracking-tight font-semibold" { "Svoote" }
                            ."size-4 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
                        }

                        a href="/" ."hover:underline" { (t!("home", locale=l)) }
                        a href="/host" ."hover:underline" { (t!("create_poll", locale=l)) }
                        a href="/data-privacy" ."hover:underline" { (t!("data_privacy", locale=l)) }
                        a href="/terms-of-service" ."hover:underline" { (t!("terms_of_service", locale=l)) }
                        a href="/cookie-policy" ."hover:underline" { (t!("cookie_policy", locale=l)) }
                        a href="/manage-cookies" ."hover:underline" { (t!("manage_cookies", locale=l)) }
                        a href="/contact" ."hover:underline" { (t!("contact", locale=l)) }
                    }
                    div ."flex justify-center gap-4" {
                        div ."flex gap-2 items-center" { div ."size-4" { (SvgIcon::Globe.render()) } (t!("language_preference", locale=l)) }
                        button onclick="setLang('en')" ."hover:underline cursor-pointer" { "English" }
                        button onclick="setLang('de')". "hover:underline cursor-pointer" { "Deutsch" }
                    }
                }
                div x-cloak x-data="{ cookiesAccepted: false }" x-show="!cookiesAccepted"
                    x-init="let local = JSON.parse(localStorage.getItem('cookiesAccepted')); cookiesAccepted = local !== null ? local : false;"
                    ."fixed max-w-2xl mx-8 bottom-12 right-0 sm:right-8 px-7 py-4 bg-white border border-cyan-600 shadow-xl"
                {
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
                        button "@click"="cookiesAccepted = true; localStorage.setItem('cookiesAccepted', 'true');"
                            ."w-full sm:w-auto px-5 py-2 bg-cyan-700 text-white text-sm font-semibold shadow-xl cursor-pointer hover:bg-cyan-600"
                            { (t!("cookie_banner_accept", locale=l)) }
                    }
                }
            }
        }
    }
}

pub fn render_header(top_right_content: Markup) -> Markup {
    return html! {
        header . "mx-6 sm:mx-14 my-7 flex justify-between" {
            a href="/" ."flex items-baseline gap-2.5 text-cyan-600" {
                span ."text-3xl tracking-tight font-semibold" { "Svoote" }
                ."size-5 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
            }
            (top_right_content)
        }
    };
}
