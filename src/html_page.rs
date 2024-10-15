use maud::{html, Markup, DOCTYPE};

use crate::{static_file, svg_icons::SvgIcon};

pub fn render_html_page(
    title: &str,
    main_content: maud::Markup,
    container_main: bool,
) -> maud::Markup {
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
                script src=(static_file::get_path("app.js")) {}
                script defer src=(static_file::get_path("cookies.js")) {}
                script defer data-domain="svoote.com" src="https://plausible.io/js/script.js" {}
            }
            body ."min-h-screen flex flex-col bg-white" {
                main
                    ."flex-1"
                    ."px-6 lg:px-10 container mx-auto"[container_main]
                {
                    (main_content)
                }

                footer ."mt-12 px-4 py-8 bg-slate-100 text-slate-900 flex justify-center items-start gap-8 md:gap-16" {
                    ."flex flex-col gap-2 text-xs" {
                        ."mb-2 font-medium text-sm" { "Polls" }
                        a href="/" ."hover:underline" {
                            "Join poll"
                        }
                        a href="/about#features" ."hover:underline" {
                            "Features"
                        }
                        a href="/about#pricing" ."hover:underline" {
                            "Pricing"
                        }
                        a href="/" ."hover:underline" {
                            "Create a poll"
                        }
                    }
                    ."flex flex-col gap-2 text-xs" {
                        ."mb-2 font-medium text-sm" { "Svoote" }
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
                        a href="#manage_cookies" ."hover:underline" {
                            "Manage cookies"
                        }
                        a href="/contact" ."hover:underline" {
                            "Contact"
                        }
                    }
                }
            }
        }
    }
}

pub fn render_header(top_right_content: Markup) -> Markup {
    return html! {
        header . "mt-8 mb-12 flex justify-between" {
            a href="/" ."flex items-baseline gap-2 text-indigo-500" {
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
