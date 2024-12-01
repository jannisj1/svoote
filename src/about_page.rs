use crate::{
    app_error::AppError,
    html_page::{self, render_header},
    svg_icons::SvgIcon,
};
use axum::response::{IntoResponse, Response};
use maud::{html, Markup};

pub async fn get_about_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page("Svoote - About", html! {
        (render_header(html!{}))
        h1 ."mt-64 mb-8 text-center text-slate-800 text-6xl font-medium" { "Modern live polling" }
        h2 ."mb-16 text-center text-slate-700 text-xl leading-8" {
            "Powerful and simple modern live polling. Free for up to 100 live users." br;
            "No login needed. No credit card required."
        }
        div ."w-fit mb-24 mx-auto px-6 py-4 flex flex-col items-center bg-slate-700 rounded-xl shadow-lg" {
            div ."mb-3 text-slate-300 text-sm" { "Want to create your own poll?" }
            a ."px-6 py-2 text-slate-900 bg-slate-50 rounded-md hover:bg-slate-300 transition" href="/" { "Start now →" }
        }
        section ."my-64" {
            (render_section("Mission", "Simple. Privacy friendly. Open source.", html! {
                ."grid md:grid-cols-2 gap-10 text-slate-700" {
                    ."flex flex-col gap-10" {
                        ."p-6 bg-white rounded-lg" {
                            ."mb-2 text-xl font-semibold flex items-center gap-2" {
                                ."size-5" { (SvgIcon::Lock.render()) }
                                "Privacy friendly"
                            }
                            ."" {
                                "To be privacy friendly, the free tier of Svoote is avaible to everyone without creating an account. "
                                "This protects your data and makes operating the website simpler. "
                                "We don't use cookies to track users, neither the poll-hosters nor the participants. "
                                "To analyze our website traffic, we use " a ."underline" href="https://plausible.io" { "Plausible" }
                                ", an EU-based privacy focused Google-analytics alternative, which does not track users accross websites."
                            }
                        }

                    }
                    ."flex flex-col gap-10" {
                        ."p-6 bg-white rounded-lg" {
                            ."mb-2 text-xl font-semibold flex items-center gap-2" {
                                ."size-5" { (SvgIcon::Image.render()) }
                                "Ad-free"
                            }
                            ."" {
                                "To be a good experience for everyone, we believe in serving Svoote ad-free, even in the free tier. "
                                "To support the operation and development of this website, you can subscribe to the Pro-version in the future (not availabe yet)."
                            }
                        }
                        ."p-6 bg-white rounded-lg" {
                            ."mb-2 text-xl font-semibold flex items-center gap-2" {
                                ."size-5" { (SvgIcon::Github.render()) }
                                "Open source"
                            }
                            ."" {
                                "Svoote is an open source website published under the GNU Affero General Public License 3 (AGPLv3). "
                                "You can check out the code, file issues or commit changes on "
                                a ."underline" href="https://github.com/jannisj1/svoote" { "Github" } "."
                            }
                        }
                    }
                }
            }))
        }
        section ."mb-64" {
            (render_section("Plans and Pricing", "Use it for free. Upgrade if you need to.", html! {
                ."flex max-sm:flex-col justify-center gap-20" {
                    (render_plan(
                        "Free",
                        "Everything you need to get started.",
                        "$0",
                        "Up to 100 live users",
                        "No account needed",
                        &[
                            "Unlimited number of polls",
                            "Up to 100 live users",
                            "Multiple choice questions",
                            "Free text questions"
                        ])
                    )
                    (render_plan(
                        "Pro",
                        "More question types, built for large audiences.",
                        "$4",
                        "Unlimited live users",
                        "Ready for large conferences",
                        &[
                            "Unlimited number of polls",
                            "Unlimited live users",
                            "Multiple choice questions",
                            "Free text questions",
                            "Image questions",
                            "Number questions"
                        ])
                    )
                }
            }))
        }
    }).into_response());
}

fn render_section(section_name: &str, heading: &str, content: Markup) -> Markup {
    return html! {
        ."max-w-screen-lg mx-auto mb-2 px-8 text-xl text-slate-600 font-medium" { (section_name) }
        ."max-w-screen-lg mx-auto bg-slate-100 px-6 py-12 rounded-lg" {
            ."mb-12 text-slate-800 text-3xl font-medium text-center leading-10" {
                (heading)
            }
            (content)
        }
    };
}

fn render_plan(
    name: &str,
    subtitle: &str,
    price: &str,
    sub_price_1: &str,
    sub_price_2: &str,
    included: &[&str],
) -> Markup {
    return html! {
        ."w-64 max-sm:w-full p-8 bg-white rounded-lg" {
            ."mb-6 text-2xl text-slate-900 font-medium tracking-tight" { (name) }
            ."mb-10 text-slate-800 tracking-wide leading-normal" { (subtitle) }
            ."mb-3 text-slate-800" {
                "Starting at"
            }
            ."mb-4 flex justify-start items-baseline gap-2" {
                ."text-4xl text-slate-900" { (price) }
                ."text-slate-500" { "USD per month" }
            }
            ."mb-10 text-sm text-slate-500 leading-2 tracking-tight" {
                (sub_price_1) br; (sub_price_2)
            }

            @if name == "Free" {
                a
                    ."mb-16 block w-fit px-5 py-3 bg-slate-100 text-slate-800 font-medium rounded-xl hover:bg-slate-200 transition"
                    href="/poll"
                {
                    "Start now"
                }
            } @else {
                ."mb-16 block w-fit px-5 py-3 bg-slate-800 text-slate-100 font-medium rounded-xl" {
                    "Coming soon"
                }
            }

            ."mb-2 text-sm text-slate-800 tracking-wide" {
                "What's included in " span ."font-medium tracking-tight" { (name) } ":"
            }

            ."flex flex-col gap-1 text-sm text-slate-800" {
                @for feature in included {
                    ."flex items-center gap-2" {
                        ."size-4 shrink-0" { (SvgIcon::Check.render()) }
                        (feature)
                    }
                }
            }
        }
    };
}
