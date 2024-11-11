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
                script defer data-domain="svoote.com" src="https://plausible.io/js/script.js" {}
                script {
                    "let colorPalette = ["
                    @for color in COLOR_PALETTE {
                        "'" (color) "',"
                    }
                    "];"
                }
            }
            body ."min-h-screen flex flex-col bg-white" {
                main ."flex-1 mx-auto w-full max-w-screen-2xl" {
                    (main_content)
                }

                footer ."mt-4 px-4 py-8 text-xs text-slate-500 flex justify-center flex-wrap gap-4" {
                    a href="/" ."hover:underline" {
                        "Home"
                    }
                    a href="/about" ."hover:underline" {
                        "About"
                    }
                    a href="/data-privacy" ."hover:underline" {
                        "Data privacy"
                    }
                    a href="/terms-of-service" ."hover:underline" {
                        "Terms of service"
                    }
                    a href="/cookie-policy" ."hover:underline" {
                        "Cookie policy"
                    }
                    a href="/contact" ."hover:underline" {
                        "Contact"
                    }
                }
            }
        }
    }
}

pub fn render_header(top_right_content: Markup) -> Markup {
    return html! {
        header . "mx-6 lg:mx-10 mt-8 mb-12 flex justify-between" {
            a href="/" ."flex items-baseline gap-2 text-slate-500" {
                span ."text-3xl tracking-tighter font-medium" { "Svoote" }
                ."size-5 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
            }
            /*."mt-1 absolute inset-0 h-full w-fit mx-auto hidden md:flex justify-center items-center gap-8" {
                a href="/" ."text-slate-700 text-sm font-medium" { "Create poll" }
                a href="/about#features" ."text-slate-700 text-sm font-medium" { "Features" }
                a href="/about#pricing" ."text-slate-700 text-sm font-medium" { "Pricing" }
                a href="/about#mission" ."text-slate-700 text-sm font-medium" { "Why Svoote?" }
            }*/
            (top_right_content)
        }
    };
}
