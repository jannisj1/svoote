use maud::{html, Markup, DOCTYPE};

use crate::{config::COLOR_PALETTE, static_file, svg_icons::SvgIcon};

pub fn render_html_page(title: &str, main_content: maud::Markup) -> maud::Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
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
                main ."flex-1 mx-auto w-full max-w-screen-xl" {
                    (main_content)
                }
                footer ."mt-4 px-4 py-8 text-xs text-slate-500 flex justify-center flex-wrap gap-4" {
                    a href="/" ."hover:underline" { "Home" }
                    a href="/about" ."hover:underline" { "About" }
                    a href="/data-privacy" ."hover:underline" { "Data privacy" }
                    a href="/terms-of-service" ."hover:underline" { "Terms of service" }
                    a href="/cookie-policy" ."hover:underline" { "Cookie policy" }
                    a href="/manage-cookies" ."hover:underline" { "Manage cookies" }
                    a href="/contact" ."hover:underline" { "Contact" }
                }
                div x-cloak x-data="{ cookiesAccepted: false }" x-show="!cookiesAccepted"
                    x-init="let local = JSON.parse(localStorage.getItem('cookiesAccepted')); cookiesAccepted = local !== null ? local : false;"
                    ."fixed mx-8 bottom-12 px-6 py-4 bg-slate-700 border shadow-xl"
                {
                    h1 ."mb-1 text-slate-100 text-xl font-semibold tracking-tight" { "Cookies" }
                    p ."mb-5 text-slate-300" {
                        "Svoote.com only uses necessary cookies. We don't use cookies to track users across sites or show ads. "
                        "For more information see our " a href="/cookie-policy" ."underline" { "Cookie Policy"} "."
                    }
                    div ."flex flex-wrap justify-end gap-4" {
                        a href="/manage-cookies" ."px-4 py-1 bg-slate-500 text-white font-bold hover:bg-slate-400" { "Customize" }
                        button "@click"="cookiesAccepted = true; localStorage.setItem('cookiesAccepted', 'true');"
                            ."px-4 py-1 bg-teal-700 text-white font-bold hover:bg-teal-600" { "Accept necessary cookies" }
                    }
                }
            }
        }
    }
}

pub fn render_header(top_right_content: Markup) -> Markup {
    return html! {
        header . "mx-6 sm:mx-14 mt-5 mb-6 flex justify-between" {
            a href="/" ."z-10 flex items-baseline gap-2.5 text-cyan-600" {
                span ."text-3xl tracking-tight font-semibold" { "Svoote" }
                ."size-5 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
            }
            (top_right_content)
        }
    };
}
