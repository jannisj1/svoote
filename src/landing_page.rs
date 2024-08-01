use std::sync::OnceLock;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use maud::{html, Markup};
use serde::Deserialize;

use crate::{
    app_error::AppError,
    html_page,
    live_item::{
        FreeTextLiveAnswers, LiveAnswers, LiveItem, MultipleChoiceLiveAnswers, COLOR_PALETTE,
    },
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    svg_icons::SvgIcon,
    word_cloud::WordCloud,
};

#[derive(Deserialize)]
pub struct GetStartPageParams {
    pub poll_id: Option<String>,
}

pub async fn get_landing_page(
    Query(params): Query<GetStartPageParams>,
) -> Result<Response, AppError> {
    let poll_id_str = params.poll_id.unwrap_or(String::new());

    if let Ok(poll_id) = poll_id_str.parse::<ShortID>() {
        if let Some(_) = LIVE_POLL_STORE.get(poll_id) {
            return Ok(Redirect::to(&format!("/p?c={}", poll_id)).into_response());
        }
    }

    return Ok(html_page::render_html_page( "Svoote", html! {
        ."container mx-auto px-4" {
            form ."mt-28 mb-48 block flex flex-col items-center" {
                ."mb-4 text-slate-700 text-sm" {
                    "Join a poll by entering the code in front:"
                }

                ."flex justify-center items-center gap-3" {
                    input
                        type="text"
                        name="poll_id"
                        value=(poll_id_str)
                        pattern="[0-9]*"
                        inputmode="numeric"
                        maxlength="6"
                        placeholder="Code"
                        ."w-32 px-6 py-2 text-slate-700 text-lg border-2 rounded-lg outline-none transition"
                        ."border-slate-300 focus:border-slate-500"[poll_id_str.is_empty()]
                        ."border-red-300 focus:border-red-500"[!poll_id_str.is_empty()];
                    button #join-btn
                        aria-label="Join poll"
                        ."relative group p-1 bg-slate-700 rounded-full hover:bg-slate-900 transition"
                        onclick="document.getElementById('join-btn').classList.add('htmx-request');"
                    {
                        ."block group-[.htmx-request]:hidden size-8 text-slate-100" { (SvgIcon::ArrowRight.render()) }
                        ."hidden size-8 group-[.htmx-request]:flex justify-center items-center" {
                            ."size-4" { (SvgIcon::Spinner.render()) }
                        }
                    }
                }
            }

            ."mb-8 text-center text-slate-800 text-6xl font-medium" {
                "Modern live polling"
            }

            ."mb-16 text-center text-slate-700 text-xl leading-8" {
                "Powerful and simple modern live polling. Free for up to 100 live users." br;
                "No login needed. No credit card required."
            }

            ."w-fit mb-24 mx-auto px-6 py-4 flex flex-col items-center bg-slate-700 rounded-xl shadow-lg" {
                ."mb-3 text-slate-300 text-sm" {
                    "Want to create your own poll?"
                }
                a
                    ."px-6 py-2 flex items-center gap-2 text-slate-900 bg-slate-50 rounded-md hover:bg-slate-300 transition"
                    href="/poll"
                {
                    "Start now"
                    ."size-4 shrink-0" { (SvgIcon::ArrowRight.render()) }
                }
            }
        }

        #features ."my-64" {
            (render_section("Features", "Get feedback from your audience in real time.", html! {
                ."grid lg:grid-cols-2 gap-16" {
                    ."p-6 bg-white rounded-lg" {
                        ."mb-7 text-2xl text-slate-700 font-medium tracking-tight flex justify-center items-center gap-2" {
                            ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                            "Multiple choice"
                        }
                        div #"demo-mc-container" hx-get="/demo_mc" hx-trigger="demoTick" hx-include="find input" {
                            (render_mc_demo(0))
                        }
                    }
                    ."p-6 bg-white rounded-lg h-[38em] lg:h-full" {
                        ."mb-7 text-2xl text-slate-700 font-medium tracking-tight flex justify-center items-center gap-2" {
                            ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                            "Free text"
                        }
                        div #"demo-ft-container" hx-get="/demo_ft" hx-trigger="demoTick" hx-include="find input" {
                            (render_ft_demo(0))
                        }
                    }
                }
            }))
        }
        script { "initStartPageDemoAnimations()" }

        #pricing ."mb-64" {
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

        #mission ."mb-64" {
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
    }, false).into_response());
}

fn render_section(section_name: &str, heading: &str, content: Markup) -> Markup {
    return html! {
        ."container mx-auto mb-2 px-8 text-xl text-slate-600 font-medium" { (section_name) }
        ."bg-slate-100 px-6 py-12 lg:rounded-lg lg:container lg:mx-auto" {
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

#[derive(Deserialize)]
pub struct GetStartPageDemoParams {
    pub index: usize,
}

pub async fn get_mc_start_page_demo(Query(params): Query<GetStartPageDemoParams>) -> Response {
    return render_mc_demo(params.index).into_response();
}

pub async fn get_ft_start_page_demo(Query(params): Query<GetStartPageDemoParams>) -> Response {
    return render_ft_demo(params.index).into_response();
}

pub fn render_mc_demo(index: usize) -> Markup {
    let stats_timeline = [
        (0, 0, 0),
        (1, 0, 0),
        (1, 1, 0),
        (1, 2, 0),
        (1, 2, 1),
        (2, 2, 1),
        (3, 2, 1),
        (4, 2, 1),
        (4, 2, 2),
        (4, 2, 3),
        (5, 2, 3),
        (6, 2, 3),
        (6, 3, 3),
        (7, 3, 3),
        (8, 3, 3),
        (8, 3, 4),
        (8, 3, 5),
        (9, 3, 5),
        (9, 4, 5),
        (10, 4, 5),
    ];

    let mut example_mc_item = LiveItem {
        question: "Is Severus Snape a good person?".to_string(),
        answers: LiveAnswers::SingleChoice(MultipleChoiceLiveAnswers {
            answers: vec![
                ("Yes".to_string(), false),
                ("No".to_string(), false),
                ("Maybe".to_string(), false),
            ],
            answer_counts: vec![
                stats_timeline[index % stats_timeline.len()].0,
                stats_timeline[index % stats_timeline.len()].1,
                stats_timeline[index % stats_timeline.len()].2,
            ],
            player_answers: Vec::new(),
        }),
    };

    return html! {
        input type="hidden" name="index" value=(index + 1);
        (example_mc_item.render_host_view())
        ."h-24" {}
        (example_mc_item.render_statistics())
    };
}

static FT_DEMO_MARKUP: OnceLock<Vec<Markup>> = OnceLock::new();

pub fn render_ft_demo(index: usize) -> Markup {
    let markup_array = FT_DEMO_MARKUP.get_or_init(|| {
        let mut example_ft_item = LiveItem {
            question: "How do you feel about the upcoming exam?".to_string(),
            answers: LiveAnswers::FreeText(FreeTextLiveAnswers {
                word_cloud: WordCloud::new(),
                player_answers: Vec::new(),
            }),
        };

        let answers = [
            "didn't learn yet",
            "well prepared",
            "stressed",
            "well prepared",
            "stressed",
            "what exam",
            "stressed",
            "don't care",
            "no time to learn",
            "no time to learn",
            "what exam",
            "well prepared",
            "well prepared",
        ];
        let mut res = Vec::new();

        for ans in answers {
            if let LiveAnswers::FreeText(ft_answers) = &mut example_ft_item.answers {
                ft_answers.word_cloud.insert(ans);
            }

            res.push(html! {
                input type="hidden" name="index" value=(res.len() + 1);
                (example_ft_item.render_host_view())
                ."h-24" {}
                (example_ft_item.render_statistics())
            });
        }

        return res;
    });

    return markup_array[index % markup_array.len()].clone();
}
