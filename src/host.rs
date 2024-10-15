use std::convert::Infallible;

use axum::{
    extract::Path,
    response::{sse, IntoResponse, Response, Sse},
};

use futures::Stream;
use maud::{html, Markup, PreEscaped};
use qrcode::{render::svg, QrCode};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt as _;
use tower_sessions::Session;

use crate::{
    app_error::AppError,
    html_page::{self, render_header},
    live_poll::{QuestionAreaState, QuestionStatisticsState},
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    svg_icons::SvgIcon,
};

pub async fn render_live_host(poll_id: ShortID) -> Result<Response, AppError> {
    Ok(html_page::render_html_page(
        "Svoote",
        html! {
            (render_header(html! {
                ."flex items-center gap-3" {
                    ."text-sm text-slate-500" {
                        "Exit poll"
                    }
                    button
                        #start-poll-btn
                        hx-post={ "/exit_poll/" (poll_id) }
                        hx-swap="none"
                        ."relative group size-12 text-slate-100 bg-red-500 rounded-full hover:bg-red-700 transition"
                    {
                        ."group-[.htmx-request]:opacity-0 flex justify-center" { ."size-6" { (SvgIcon::X.render()) } }
                        ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                            ."size-4" { (SvgIcon::Spinner.render()) }
                        }
                    }
                }
            }))
            ."flex flex-col gap-16" {
                div hx-ext="sse" sse-connect={"/sse/host_question/" (poll_id) } sse-close="close" {
                    div sse-swap="update" { (render_sse_loading_spinner()) }
                }
                div hx-ext="sse" sse-connect={"/sse/host_results/" (poll_id) } sse-close="close"  {
                    div sse-swap="update" { }
                }
            }
        },
        true,
    )
    .into_response())
}

pub async fn get_sse_host_question(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let lq = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = lq.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let updates = lq.lock().unwrap().ch_question_state.clone();

    let stream = WatchStream::new(updates)
        .map(move |state| match state {
            QuestionAreaState::None => sse::Event::default()
            .event("update")
            .data(html! {}.into_string()),
            QuestionAreaState::JoinCode(poll_id) => sse::Event::default()
            .event("update")
            .data(html! {
                @let join_url = format!("https://svoote.com/p?c={}", poll_id);
                @let join_qr_code_svg = QrCode::new(&join_url)
                    .map(|qr|
                        qr.render()
                        .min_dimensions(200, 200)
                        .quiet_zone(false)
                        .dark_color(svg::Color("#000000"))
                        .light_color(svg::Color("#FFFFFF"))
                        .build()
                    );

                ."mt-8 flex justify-center items-center gap-20" {
                    ."w-lg flex justify-center" {
                        (PreEscaped(join_qr_code_svg.unwrap_or("Error generating QR-Code.".to_string())))
                    }
                    ."flex flex-col items-center gap-2" {
                        ."text-lg font-bold text-indigo-600 tracking-tight" {
                            "Join now"
                        }
                        ."size-5"{ (SvgIcon::Spinner.render()) }
                    }
                    ."text-center" {
                        ."mb-2 text-5xl tracking-wider font-bold text-slate-700" {
                            (poll_id)
                        }
                        ."text-sm text-slate-700" {
                            "Enter on " a ."text-indigo-500 underline" href=(join_url) { "svoote.com" }
                        }
                    }
                }
                button
                    hx-post={ "/next_question/" (poll_id) }
                    hx-swap="none"
                    ."relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                {
                    ."group-[.htmx-request]:opacity-0 flex items-center gap-2" {
                        "Start first item"
                        ."size-4" { (SvgIcon::ArrowRight.render()) }
                    }
                    ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                        ."size-4" { (SvgIcon::Spinner.render()) }
                    }
                }
            }.into_string()),
            QuestionAreaState::Item { item_idx: question_idx, is_last_question } => {
                sse::Event::default()
                .event("update")
                .data(
                    html! {
                        ."mb-10 grid grid-cols-3 items-center" {
                            @if question_idx == 0 {
                                div {}
                            } @else {
                                button
                                    hx-post={ "/previous_question/" (poll_id) }
                                    hx-swap="none"
                                    ."justify-self-start flex gap-2 items-center text-slate-500 text-sm"
                                {
                                    ."size-4" { (SvgIcon::ArrowLeft.render()) }
                                    "Previous"
                                }
                            }

                            ."text-center text-sm text-slate-500" {
                                "Question " (question_idx + 1)
                            }

                            button
                                hx-post={ "/next_question/" (poll_id) }
                                hx-swap="none"
                                ."justify-self-end relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                            {
                                ."group-[.htmx-request]:opacity-0 flex items-center gap-2" {
                                    @if !is_last_question {
                                        "Next"
                                        ."size-4" { (SvgIcon::ArrowRight.render()) }
                                    } @else {
                                        "End poll"
                                    }
                                }
                                ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                                    ."size-4" { (SvgIcon::Spinner.render()) }
                                }
                            }
                        }
                        (lq.lock().unwrap().items[question_idx].render_host_view())
                    }.into_string()
                )
            },
            QuestionAreaState::PollFinished => {
                sse::Event::default()
                .event("update")
                .data(
                    html! {
                        ."my-24 flex flex-col text-sm text-slate-500 text-center" {
                            ."" { "This poll has no more items. Thank you for using svoote.com" }
                            a
                                href="/"
                                ."mt-4 underline cursor-pointer hover:text-slate-700 transition"
                            {
                                "Back to editing this poll"
                            }
                        }
                    }.into_string()
                )
            },
            QuestionAreaState::CloseSSE => {
                sse::Event::default()
                .event("close")
                .data("")
            }
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
}

pub async fn get_live_statistics(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = live_poll.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let ch_question_statistics = live_poll
        .lock()
        .unwrap()
        .ch_question_statistics_recv
        .clone();

    let stream = WatchStream::new(ch_question_statistics)
        .map(move |statistics| match statistics {
            QuestionStatisticsState::None => sse::Event::default().event("update").data(""),
            QuestionStatisticsState::Item(item_idx) => sse::Event::default().event("update").data(
                live_poll.lock().unwrap().items[item_idx]
                    .render_statistics()
                    .into_string(),
            ),
            QuestionStatisticsState::CloseSSE => sse::Event::default().event("close").data(""),
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
}

pub async fn get_sse_leaderboard(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = live_poll.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let update_channel = live_poll.lock().unwrap().ch_players_updated_recv.clone();

    let stream = WatchStream::new(update_channel)
        .map(move |_| {
            let live_poll = live_poll.lock().unwrap();

            html! {
                ."mt-10 mb-4 flex gap-2 items-center" {
                    ."size-5 text-amber-500" { (SvgIcon::Crown.render()) }
                    ."text-xl font-medium text-slate-600" { "Leaderboard" }
                }
                ."mb-2 text-sm text-slate-600 tracking-tight" {
                    (live_poll.players.len())
                    " player" (if live_poll.players.len() != 1 { "s" } else { "" })
                }
                ."max-h-96 overflow-scroll" {
                    @for (_player_index, player) in live_poll.players.iter().enumerate() {
                        ."flex justify-between" {
                            ."flex items-center text-slate-900 gap-1" {
                                ."size-5" { (player.get_avatar_svg()) }
                                (player.get_name())
                            }
                            ."text-slate-600 font-medium tracking-tight" { (0) }
                        }
                    }
                }
            }
        })
        .map(|html| {
            sse::Event::default()
                .event("update")
                .data(html.into_string())
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
}

pub async fn post_next_question(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Response, AppError> {
    let lq = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = lq.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let next_question_send = lq.lock().unwrap().ch_next_question.clone();
    let _ = next_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_previous_question(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Response, AppError> {
    let lq = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = lq.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let previous_question_send = lq.lock().unwrap().ch_previous_question.clone();
    let _ = previous_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_exit_poll(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Response, AppError> {
    let lq = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let auth_token = lq.lock().unwrap().host_auth_token.clone();
    auth_token.verify(&session).await?;

    let exit_poll_send = lq.lock().unwrap().ch_exit_poll.clone();
    let _ = exit_poll_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub fn render_sse_loading_spinner() -> Markup {
    html! {
        ."h-64 flex items-center justify-center" {
            ."size-4" { (SvgIcon::Spinner.render()) }
        }
    }
}
