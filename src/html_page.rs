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
                meta name="description" content="Svoote is the fastest way to create free live polls. No account needed.";
                meta name="google-site-verification" content="43Qm55o4cXLfJSwfb_7gvXUUFYiYzs7zvKqUX46pk1c";
                title { (title) }
                @if cfg!(debug_assertions) {
                    script src="https://cdn.tailwindcss.com" {}
                }
                link rel="stylesheet" href=(static_file::get_path("bundle.css"));
                script defer src=(static_file::get_path("app.js")) {}
                link rel="icon" type="image/png" href="/img/svoote_icon.png";
                @if let Ok(domain) = std::env::var("PLAUSIBLE_DOMAIN") {
                    script defer data-domain=(domain) src="https://plausible.io/js/script.js" {}
                }
                script {
                    "let colorPalette = ["
                    @for color in COLOR_PALETTE {
                        "'" (color) "',"
                    }
                    "];"
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
                        a href="/" ."hover:underline" { "Home" }
                        a href="/host" ."hover:underline" { "Create Poll" }
                        a href="/data-privacy" ."hover:underline" { "Data Privacy" }
                        a href="/terms-of-service" ."hover:underline" { "Terms of Service" }
                        a href="/cookie-policy" ."hover:underline" { "Cookie Policy" }
                        a href="/manage-cookies" ."hover:underline" { "Manage Cookies" }
                        a href="/contact" ."hover:underline" { "Contact" }
                    }
                    div ."flex justify-center gap-4" {
                        div ."flex gap-2 items-center" { div ."size-4" { (SvgIcon::Globe.render()) } "Language preference:"}
                        button onclick="setLang('en')" ."hover:underline" { "English" }
                        button onclick="setLang('de')". "hover:underline" { "Deutsch" }
                    }
                }
                div x-cloak x-data="{ cookiesAccepted: false }" x-show="!cookiesAccepted"
                    x-init="let local = JSON.parse(localStorage.getItem('cookiesAccepted')); cookiesAccepted = local !== null ? local : false;"
                    ."fixed max-w-2xl mx-8 bottom-12 right-8 px-7 py-5 bg-white border border-cyan-600 shadow-xl"
                {
                    h1 ."mb-1 text-slate-700 text-xl font-semibold tracking-tight" { "Cookies" }
                    p ."mb-5 text-slate-500" {
                        "Svoote.com only uses necessary cookies. We don't use cookies to track users across sites or show ads. "
                        "For more information see our " a href="/cookie-policy" ."underline" { "Cookie Policy"} "."
                    }
                    div ."flex flex-wrap justify-end gap-4" {
                        a href="/manage-cookies" ."px-4 py-1 bg-slate-200 text-slate-700 font-semibold hover:bg-slate-300" { "Customize" }
                        button "@click"="cookiesAccepted = true; localStorage.setItem('cookiesAccepted', 'true');"
                            ."px-4 py-1 bg-cyan-700 text-white font-semibold shadow-xl hover:bg-cyan-600" { "Accept necessary cookies" }
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
