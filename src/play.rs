use crate::{
    app_error::AppError,
    auth_token::AuthToken,
    html_page,
    live_item::LiveAnswers,
    live_poll::{QuestionAreaState, LIVE_POLL_PARTICIPANT_LIMIT},
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    svg_icons::SvgIcon,
};
use axum::{
    extract::{Form, Path, Query},
    response::{sse, IntoResponse, Response, Sse},
};

use futures::Stream;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::convert::Infallible;
use tokio::time::{Duration, Instant};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt as _;
use tower_sessions::Session;

pub const MAX_FREE_TEXT_ANSWERS: usize = 3;

#[derive(Deserialize)]
pub struct PlayPageParams {
    pub c: ShortID,
}

pub async fn get_play_page(
    Query(params): Query<PlayPageParams>,
    session: Session,
) -> Result<Response, AppError> {
    let live_poll = match LIVE_POLL_STORE.get(params.c) {
        Some(lq) => lq,
        None => {
            return Ok(
                html_page::render_html_page("Svoote", render_poll_finished(), true).into_response(),
            )
        }
    };

    let auth_token = AuthToken::get_or_create(&session).await?;
    let player_name = live_poll.lock().unwrap().join(&auth_token);

    Ok(html_page::render_html_page(

        "Svoote",
        html! {
            @if let Some(player_name) = player_name {
                div hx-ext="sse" sse-connect={ "/sse/play/" (params.c) } sse-close="close" {
                    div sse-swap="update" { (crate::host::render_sse_loading_spinner()) }
                }

                @if live_poll.lock().unwrap().leaderboard_enabled {
                    ."mt-4 mb-16 text-slate-500 text-sm" {
                        "Leaderboard is enabled. Playing as `" (player_name) "`."
                    }
                }
            } @else {
                ."my-36 text-center text-slate-500" {
                    "The participant limit for this poll was reched (" (LIVE_POLL_PARTICIPANT_LIMIT) " participants)."
                }
            }
        }, true
    )
    .into_response())
}

pub async fn get_sse_play(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let updates = live_poll.lock().unwrap().ch_question_state.clone();
    let auth_token = AuthToken::get_or_create(&session).await?;
    let player_index = {
        let live_poll = live_poll.lock().unwrap();
        live_poll
            .players
            .get(&auth_token.token)
            .ok_or(AppError::BadRequest(
                "This player did not join the poll yet".to_string(),
            ))?
            .player_index
    };

    let stream = WatchStream::new(updates)
        .map(move |state| match state {
            QuestionAreaState::None => sse::Event::default()
            .event("update")
            .data(""),
            QuestionAreaState::Item { item_idx, is_last_question: _ } => {
                let live_poll = live_poll.lock().unwrap();

                sse::Event::default()
                .event("update")
                .data(
                    html! {
                        ."mb-2 flex gap-6 text-sm text-slate-500" {
                            "Question " (item_idx + 1)
                            @match &live_poll.items[item_idx].answers {
                                LiveAnswers::SingleChoice(_mc_answers) => {
                                    ."flex gap-1 items-center" {
                                        ."size-4" { (SvgIcon::CheckSquare.render()) }
                                        "Multiple choice"
                                    }
                                }
                                LiveAnswers::FreeText(_ft_answers) => {
                                    ."flex gap-1 items-center" {
                                        ."size-4" { (SvgIcon::Edit3.render()) }
                                        "Free text - up to " (MAX_FREE_TEXT_ANSWERS) " answers"
                                    }
                                }
                            }
                        }
                        ."mb-4 text-lg text-slate-700" {
                            (live_poll.items[item_idx].question)
                        }
                        @match &live_poll.items[item_idx].answers {
                            LiveAnswers::SingleChoice(mc_answers) => {
                                @let current_mc_answer = &mc_answers.player_answers[player_index];
                                form ."block w-full" {
                                    @for (answer_idx, (answer_txt, _is_correct)) in mc_answers.answers.iter().enumerate() {
                                        label
                                            onclick="let e = document.getElementById('submit-btn'); if (e !== null) e.disabled = false;"
                                            .{
                                                "block p-2 mb-4 text-center text-base text-slate-700 "
                                                "rounded-lg ring-2 ring-slate-500 "
                                                "hover:ring-indigo-500 "
                                                "has-[:checked]:ring-4 has-[:checked]:ring-indigo-500 "
                                                "cursor-pointer transition duration-100 "
                                            } {
                                            (answer_txt)
                                            input ."hidden" type="radio" name="answer_idx" value=(answer_idx)
                                                required
                                                disabled[current_mc_answer.is_some()]
                                                checked[current_mc_answer.is_some_and(|ans| ans == answer_idx)];
                                        }
                                    }
                                    ."flex justify-center mt-6" {
                                        @if current_mc_answer.is_none() {
                                            button #"submit-btn"
                                                hx-post={ "/submit_mc_answer/" (poll_id) }
                                                hx-target="this"
                                                hx-swap="outerHTML"
                                                disabled
                                                ."relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                                            {
                                                ."group-[.htmx-request]:opacity-0" { "Submit answer" }
                                                ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                                                    ."size-4" { (SvgIcon::Spinner.render()) }
                                                }
                                            }
                                        } @else {
                                            (render_answer_submitted_text())
                                        }
                                    }
                                }
                            },
                            LiveAnswers::FreeText(ft_answers) => {
                                (render_free_text_form(poll_id, &ft_answers.player_answers[player_index]))
                            }
                        }
                        ."my-36 text-sm text-slate-500 text-center" {
                            p ."mb-2" {
                                "This poll is powered by svoote.com"
                            }
                            p ."" {
                                "Svoote does not assume responsibility for the polls created on this website."
                            }
                        }
                    }.into_string()
                )
            },
            QuestionAreaState::PollFinished => {
                sse::Event::default()
                .event("update")
                .data(
                    render_poll_finished().into_string()
                )
            }
            QuestionAreaState::CloseSSE => {
                sse::Event::default()
                .event("close")
                .data("")
            }
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
}

fn render_answer_submitted_text() -> Markup {
    html! {
        ."text-slate-700" { "Your answer has been submitted." }
    }
}

#[derive(Deserialize)]
pub struct PostMCAnswerForm {
    pub answer_idx: usize,
}

pub async fn post_mc_answer(
    Path(poll_id): Path<ShortID>,
    session: Session,
    Form(form): Form<PostMCAnswerForm>,
) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let mut live_poll = live_poll.lock().unwrap();

    let player_idx = live_poll.get_player(&auth_token)?.player_index;

    if let LiveAnswers::SingleChoice(mc_answers) = &mut live_poll.get_current_item().answers {
        if form.answer_idx >= mc_answers.answers.len() {
            return Err(AppError::BadRequest("answer_idx out of bounds".to_string()));
        }

        if mc_answers.player_answers[player_idx].is_some() {
            return Err(AppError::BadRequest(
                "Already submitted an answers".to_string(),
            ));
        }

        mc_answers.player_answers[player_idx] = Some(form.answer_idx);
        mc_answers.answer_counts[form.answer_idx] += 1;

        if mc_answers.answers[form.answer_idx].1 {
            let mut elapsed = Instant::now() - live_poll.current_item_start_time;
            if elapsed > Duration::from_secs(60) {
                elapsed = Duration::from_secs(60);
            }

            let fraction_points = (60_000 - elapsed.as_millis()) as f32 / 60_000f32;
            live_poll.get_player(&auth_token)?.score += 50 + (fraction_points * 50f32) as u32;

            let _ = live_poll.ch_players_updated_send.send(());
        }
    } else {
        return Err(AppError::BadRequest(
            "This is not a MC-question".to_string(),
        ));
    }

    live_poll
        .ch_question_statistics_send
        .send_if_modified(|_stats| true);

    return Ok(html! {
        (render_answer_submitted_text())
        script { (PreEscaped(r#"document.querySelectorAll('input[type="radio"]').forEach((e) => { e.disabled = true });"#)) }
    }
    .into_response());
}

#[derive(Deserialize)]
pub struct PostFreeTextAnswerForm {
    pub free_text_answer: String,
}

pub async fn post_free_text_answer(
    Path(poll_id): Path<ShortID>,
    session: Session,
    Form(form_data): Form<PostFreeTextAnswerForm>,
) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let mut live_poll = live_poll.lock().unwrap();
    let player_idx = live_poll.get_player(&auth_token)?.player_index;

    let response = if let LiveAnswers::FreeText(ft_answers) =
        &mut live_poll.get_current_item().answers
    {
        if ft_answers.player_answers[player_idx].len() >= MAX_FREE_TEXT_ANSWERS {
            return Err(AppError::BadRequest(format!(
                "Already submitted the maximum number of free text answers ({})",
                MAX_FREE_TEXT_ANSWERS
            )));
        }

        ft_answers.player_answers[player_idx].push(form_data.free_text_answer.clone());
        ft_answers.word_cloud.insert(&form_data.free_text_answer);

        Ok(render_free_text_form(poll_id, &ft_answers.player_answers[player_idx]).into_response())
    } else {
        return Err(AppError::BadRequest(
            "This is not a free text answer".to_string(),
        ));
    };

    live_poll
        .ch_question_statistics_send
        .send_if_modified(|_stats| {
            return true;
        });

    return response;
}

fn render_poll_finished() -> Markup {
    html! {
        ."my-36 text-center text-slate-500" {
            "This poll is finished." br;
            "Thank you for participating on svoote.com."
        }
    }
}

fn render_free_text_form(poll_id: ShortID, answers: &[String]) -> Markup {
    return html! {
        form #free-text-form ."flex flex-col items-center" {
            ."w-full mb-2" {
                @for (i, answer) in answers.iter().enumerate() {
                    ."mb-2 text-lg text-slate-700" {
                        (i + 1) ". " (answer)
                    }
                }
            }
            @if answers.len() < MAX_FREE_TEXT_ANSWERS {
                input type="text" name="free_text_answer" autofocus
                    ."w-full mb-4 text-lg px-2 py-1 border-2 border-slate-500 rounded-lg outline-none hover:border-indigo-600 focus:border-indigo-600 transition"
                    placeholder="Answer";
                button
                    hx-post={ "/submit_free_text_answer/" (poll_id) }
                    hx-target="#free-text-form"
                    hx-swap="outerHTML"
                    ."relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                {
                    ."group-[.htmx-request]:opacity-0" { "Submit answer" }
                    ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                        ."size-4" { (SvgIcon::Spinner.render()) }
                    }
                }
            }
        }

    };
}
