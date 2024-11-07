use std::convert::Infallible;

use axum::{
    extract::Path,
    response::{sse, IntoResponse, Response, Sse},
};

use axum_extra::extract::CookieJar;
use futures::Stream;
use maud::{html, Markup};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt as _;

use crate::{
    app_error::AppError,
    html_page::{self, render_header},
    live_poll::{QuestionAreaState, QuestionStatisticsState},
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
    svg_icons::SvgIcon,
};

pub async fn render_live_host(poll_id: ShortID) -> Result<Response, AppError> {
    Ok(html_page::render_html_page(
        "Svoote",
        html! {
            (render_header(html! {}))
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

pub async fn get_sse_slides(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let updates = live_poll.lock().unwrap().ch_question_state.clone();

    let stream = WatchStream::new(updates)
        .filter_map(move |state| match state {
            QuestionAreaState::Empty => Some(
                sse::Event::default()
                    .event("update")
                    .data(html! {}.into_string()),
            ),
            QuestionAreaState::Slide(slide_index) => {
                let current_participant_count =
                    live_poll.lock().unwrap().get_current_participant_count();
                Some(
                    sse::Event::default().event("update").data(
                        live_poll.lock().unwrap().items[slide_index]
                            .render_host_view(poll_id, slide_index, current_participant_count)
                            .into_string(),
                    ),
                )
            }
            QuestionAreaState::PollFinished => None,
            QuestionAreaState::CloseSSE => Some(sse::Event::default().event("close").data("")),
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}

pub async fn get_sse_statistics(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let ch_question_statistics = live_poll
        .lock()
        .unwrap()
        .ch_question_statistics_recv
        .clone();

    let stream = WatchStream::new(ch_question_statistics)
        .map(move |statistics| match statistics {
            QuestionStatisticsState::Empty => sse::Event::default().event("update").data(""),
            QuestionStatisticsState::Slide(slide_index) => {
                sse::Event::default().event("update").data(
                    live_poll.lock().unwrap().items[slide_index]
                        .render_statistics()
                        .into_string(),
                )
            }
            QuestionStatisticsState::CloseSSE => sse::Event::default().event("close").data(""),
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}

pub async fn get_sse_leaderboard(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

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

pub async fn post_next_slide(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let next_question_send = live_poll.lock().unwrap().ch_next_question.clone();
    let _ = next_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_previous_slide(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let previous_question_send = live_poll.lock().unwrap().ch_previous_question.clone();
    let _ = previous_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_exit_poll(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let exit_poll_send = live_poll.lock().unwrap().ch_exit_poll.clone();
    let _ = exit_poll_send.send(()).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    return Ok("Poll exited".into_response());
}

pub fn render_sse_loading_spinner() -> Markup {
    html! {
        ."h-64 flex items-center justify-center" {
            ."size-4" { (SvgIcon::Spinner.render()) }
        }
    }
}

pub async fn get_sse_user_counter(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let update_channel = live_poll.lock().unwrap().ch_players_updated_recv.clone();

    let mut last_send_player_count = None;

    let stream = WatchStream::new(update_channel)
        .filter_map(move |_| {
            let live_poll = live_poll.lock().unwrap();

            if last_send_player_count.is_some_and(|count| count == live_poll.players.len()) {
                None
            } else {
                last_send_player_count = Some(live_poll.players.len());
                Some(html! { ."text-slate-600 text-lg" { (live_poll.players.len()) } })
            }
        })
        .map(|html| {
            sse::Event::default()
                .event("update")
                .data(html.into_string())
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}
