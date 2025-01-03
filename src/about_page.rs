use crate::{
    app_error::AppError,
    html_page::{self, render_header},
    svg_icons::SvgIcon,
};
use axum::response::{IntoResponse, Response};
use maud::html;

pub async fn get_about_page() -> Result<Response, AppError> {
    return Ok(html_page::render_html_page("Svoote - About", html! {
        (render_header(html!{
            a href="/p" ."text-slate-500" { "Join presentation →" }
        }))
        div ."mt-24 mx-6 sm:mx-14" {
            div ."max-w-2xl mx-auto" {
                h1 ."mb-8 text-center text-slate-800 text-5xl font-bold" { "Easy to use, cheap and privacy-friendly Mentimeter alternative" }
                h2 ."mb-8 text-center text-slate-500 text-xl leading-8" {
                    "Svoote is intuitive, lightweight and open source live polling. "
                    "Host unlimited numbers of live presentations for up to 100 participants without creating an account. "
                    "Made and hosted in the EU 🇪🇺"
                }
            }
            div ."mb-32 flex justify-center" {
                a ."px-6 py-4 text-slate-50 text-lg font-medium bg-indigo-600 rounded-md hover:bg-indigo-700" href="/"
                { "Create presentation" }
            }
            div ."mx-auto max-w-screen-lg" {
                h3 ."mb-10 text-center text-slate-700 text-4xl font-bold" { "Why Svoote?" }
                section ."mb-32 grid md:grid-cols-2 gap-10 text-slate-700" {
                    div ."flex flex-col gap-10" {
                        div ."p-6 bg-white rounded-lg border shadow" {
                            h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                                { ."size-5" { (SvgIcon::Lock.render()) } "Privacy friendly" }
                            p ."" {
                                "To be privacy friendly, the free tier of Svoote is avaible to everyone without creating an account. "
                                "This protects your data and makes operating the website simpler. "
                                "We don't use cookies to track users, neither the poll-hosters nor the participants. "
                                "To analyze our website traffic, we use " a ."underline" href="https://plausible.io" { "Plausible" }
                                ", an EU-based privacy focused Google-analytics alternative, which does not track users accross websites."
                            }
                        }

                    }
                    div ."flex flex-col gap-10" {
                        div ."p-6 bg-white rounded-lg border shadow" {
                            h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                                { ."size-5" { (SvgIcon::Image.render()) } "Ad-free" }
                            p ."" {
                                "To be a good experience for everyone, we believe in serving Svoote ad-free, even in the free tier. "
                                "To support the operation and development of this website, you can subscribe to the Pro-version in the future (not availabe yet)."
                            }
                        }
                        ."p-6 bg-white rounded-lg border shadow" {
                            h4 ."mb-2 text-xl font-semibold flex items-center gap-2"
                                { ."size-5" { (SvgIcon::Github.render()) } "Open source" }
                            p ."" {
                                "Svoote is an open source website published under the GNU Affero General Public License 3 (AGPLv3). "
                                "You can check out the code, file issues or commit changes on "
                                a ."underline" href="https://github.com/jannisj1/svoote" { "Github" } "."
                            }
                        }
                    }
                }
                h3 ."mb-10 text-center text-slate-700 text-4xl font-bold" { "Plans and pricing" }
                section ."mb-8 flex justify-center gap-10 sm:gap-20 flex-wrap" {
                    div ."w-64 p-8 bg-white rounded-lg border shadow" {
                        ."mb-6 text-2xl text-slate-900 font-medium tracking-tight" { "Free" }
                        ."mb-10 text-slate-800 tracking-wide leading-normal" { "Everything you need to get started." }
                        //."mb-3 text-slate-800" { "Starting at" }
                        ."mb-4 flex justify-start items-baseline gap-2"
                            { ."text-4xl text-slate-900" { "$0" } ."text-slate-500" { "USD per month" } }
                        ."mb-10 text-sm text-slate-500 leading-2 tracking-tight"
                            { "Up to 100 live users" br; "No account needed" }
                        a ."mb-16 block w-fit px-5 py-3 bg-slate-100 text-slate-800 font-medium rounded-xl hover:bg-slate-200 transition" href="/"
                            { "Start now" }
                        div ."mb-2 text-sm text-slate-800 tracking-wide"
                            { "What's included in " span ."font-medium tracking-tight" { "Free" } ":" }

                        ul ."flex flex-col gap-1 text-sm text-slate-800" {
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Unlimited number of polls" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Up to 100 live users" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Multiple choice slides" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Word cloud slides" }
                        }
                    }
                    div ."w-64 p-8 bg-white rounded-lg border shadow" {
                        ."mb-6 text-2xl text-slate-900 font-medium tracking-tight" { "Pro" }
                        ."mb-10 text-slate-800 tracking-wide leading-normal" { "More slide templates, built for large audiences." }
                        //."mb-3 text-slate-800" { "Starting at" }
                        ."mb-4 flex justify-start items-baseline gap-2"
                            { ."text-4xl text-slate-900" { "$4" } ."text-slate-500" { "USD per month" } }
                        ."mb-10 text-sm text-slate-500 leading-2 tracking-tight"
                            { "Unlimited live users" br; "Ready for large conferences" }
                        ."mb-16 block w-fit px-5 py-3 bg-slate-800 text-slate-100 font-medium rounded-xl"
                            { "Not available yet" }
                        div ."mb-2 text-sm text-slate-800 tracking-wide"
                            { "What's included in " span ."font-medium tracking-tight" { "Pro" } ":" }

                        ul ."flex flex-col gap-1 text-sm text-slate-800" {
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Unlimited number of polls" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Unlimited live users" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Multiple choice slides" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Word cloud slides" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Image slides" }
                            li ."flex items-center gap-2" { ."size-4" { (SvgIcon::Check.render()) } "Number range slides" }
                        }
                    }
                }
            }
        }
    }).into_response());
}
