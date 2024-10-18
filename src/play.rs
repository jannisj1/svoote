use crate::{
    app_error::AppError,
    auth_token::AuthToken,
    config::{LIVE_POLL_PARTICIPANT_LIMIT, MAX_FREE_TEXT_ANSWERS},
    html_page::{self, render_header},
    live_item::LiveAnswers,
    live_poll::QuestionAreaState,
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
use smartstring::{Compact, SmartString};
use std::convert::Infallible;
use std::fmt::Write;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt as _;
use tower_sessions::Session;

// These awesome SVG-avatars were obtained from dicebear.com (Adventurer Neutral by Lisa Wischofsky)
// They are published under the CC BY 4.0 license (https://creativecommons.org/licenses/by/4.0/)
const AVATARS: &[(&'static str, &'static str)] = &[
    /* ("Zoey", include_str!("../avatars/zoey.svg")),
    ("Tigger", include_str!("../avatars/tigger.svg")),
    ("Bubba", include_str!("../avatars/bubba.svg")),
    ("Spooky", include_str!("../avatars/spooky.svg")),
    ("Sadie", include_str!("../avatars/sadie.svg")),
    ("George", include_str!("../avatars/george.svg")),
    ("Zoe", include_str!("../avatars/zoe.svg")),
    ("Baby", include_str!("../avatars/baby.svg")),
    ("Sammy", include_str!("../avatars/sammy.svg")),
    ("Cookie", include_str!("../avatars/cookie.svg")),
    ("Lola", include_str!("../avatars/lola.svg")),
    ("Snickers", include_str!("../avatars/snickers.svg")),
    ("Oliver", include_str!("../avatars/oliver.svg")),
    ("Willow", include_str!("../avatars/willow.svg")),
    ("Whiskers", include_str!("../avatars/whiskers.svg")),
    ("Samantha", include_str!("../avatars/samantha.svg")),
    ("Cuddles", include_str!("../avatars/cuddles.svg")),
    ("Sassy", include_str!("../avatars/sassy.svg")),
    ("Callie", include_str!("../avatars/callie.svg")),
    ("Ginger", include_str!("../avatars/ginger.svg")),*/
    ("Rascal", include_str!("../avatars/rascal_square.svg")),
    ("Chester", include_str!("../avatars/chester_square.svg")),
    ("Coco", include_str!("../avatars/coco_square.svg")),
    ("Bella", include_str!("../avatars/bella_square.svg")),
    ("Gizmo", include_str!("../avatars/gizmo_square.svg")),
    ("Kitty", include_str!("../avatars/kitty_square.svg")),
    ("Daisy", include_str!("../avatars/daisy_square.svg")),
    ("Angel", include_str!("../avatars/angel_square.svg")),
    ("Bubba", include_str!("../avatars/bubba_square.svg")),
    ("Boots", include_str!("../avatars/boots_square.svg")),
    ("Patches", include_str!("../avatars/patches_square.svg")),
    ("Simon", include_str!("../avatars/simon_square.svg")),
    ("Sugar", include_str!("../avatars/sugar_square.svg")),
    ("Gracie", include_str!("../avatars/gracie_square.svg")),
    ("Princess", include_str!("../avatars/princess_square.svg")),
    ("Dusty", include_str!("../avatars/dusty_square.svg")),
    ("Luna", include_str!("../avatars/luna_square.svg")),
    ("Baby", include_str!("../avatars/baby_square.svg")),
    ("Milo", include_str!("../avatars/milo_square.svg")),
    ("Jasmine", include_str!("../avatars/jasmine_square.svg")),
];

pub struct Player {
    generated_name: SmartString<Compact>,
    custom_name: Option<SmartString<Compact>>,
    avatar_index: usize,
}

impl Player {
    pub fn new(player_index: usize) -> Self {
        let avatar_index = player_index % AVATARS.len();
        let duplicate_name_number = ((player_index - avatar_index) / AVATARS.len()) + 1;
        let mut generated_name = SmartString::<Compact>::new();

        if duplicate_name_number >= 2 {
            let _ = write!(
                generated_name,
                "{} ({})",
                AVATARS[avatar_index].0, duplicate_name_number
            );
        } else {
            generated_name.push_str(AVATARS[avatar_index].0);
        }

        return Self {
            generated_name,
            custom_name: None,
            avatar_index,
        };
    }

    pub fn get_name<'a>(&'a self) -> &'a SmartString<Compact> {
        return match &self.custom_name {
            Some(name) => name,
            None => &self.generated_name,
        };
    }

    pub fn get_generated_name<'a>(&'a self) -> &'a SmartString<Compact> {
        return &self.generated_name;
    }

    pub fn get_avatar_svg(&self) -> PreEscaped<&'static str> {
        return PreEscaped(AVATARS[self.avatar_index].1);
    }
}

#[derive(Deserialize)]
pub struct PlayPageParams {
    pub c: ShortID,
}

pub async fn get_play_page(
    Query(params): Query<PlayPageParams>,
    session: Session,
) -> Result<Response, AppError> {
    let live_poll = match LIVE_POLL_STORE.get(params.c) {
        Some(live_poll) => live_poll,
        None => {
            return Ok(
                html_page::render_html_page("Svoote", render_poll_finished(), true).into_response(),
            )
        }
    };

    let auth_token = AuthToken::get_or_create(&session).await?;
    let mut live_poll = live_poll.lock().unwrap();

    return Ok(html_page::render_html_page(
        "Svoote",
        match live_poll.get_or_create_player(&auth_token) {
            Some(player_index) => {
                let player = live_poll.get_player(player_index);
                html! {
                    (render_header(html! {
                        button
                            ."px-5 py-2 flex items-center gap-2.5 bg-slate-700 hover:bg-slate-800 rounded-full transition"
                            onclick="document.getElementById('participant-dialog').showModal()"
                        {
                            ."text-slate-300" {
                                @if live_poll.leaderboard_enabled {
                                    (player.get_name())
                                } @else {
                                    "Anonymous"
                                }
                            }
                            ."size-8 text-slate-300" {
                                @if live_poll.leaderboard_enabled {
                                    (player.get_avatar_svg())
                                } @else {
                                    (SvgIcon::User.render())
                                }
                            }
                        }
                        dialog
                            #participant-dialog
                            ."w-2xl"
                        {
                            ."mb-2 flex justify-end" {
                                button
                                    onclick="document.getElementById('participant-dialog').close()"
                                    ."size-5 text-red-500"
                                { (SvgIcon::X.render()) }
                            }
                            ."" { "Name" }
                            /*input type="text"
                                name="player_name"
                                ."px-4 py-1.5 flex-1 text-slate-900 font-medium bg-slate-100 rounded-lg"
                                hx-put={ "/poll/item/" (item_idx) "/text" }
                                hx-trigger="input changed delay:300ms"
                                "hx-on::before-request"="bindSavingIndicator();"
                                "hx-on::after-request"="freeSavingIndicator();"
                                maxlength="32"
                                placeholder="Enter question text"
                                onkeydown={ "onkeydownMCAnswer(this, event, " (item_idx) ");"}
                                value=(item.question);*/
                        }
                    }))
                    div hx-ext="sse" sse-connect={ "/sse/play/" (params.c) } sse-close="close" {
                        div sse-swap="update" { (crate::host::render_sse_loading_spinner()) }
                    }
                }
            }
            None => {
                html! {
                    ."my-36 text-center text-slate-500" {
                        "The participant limit for this poll was reached (" (LIVE_POLL_PARTICIPANT_LIMIT) " participants)."
                    }
                }
            }
        },
        true
    ).into_response());
}

pub async fn get_sse_play(
    Path(poll_id): Path<ShortID>,
    session: Session,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let updates = live_poll.lock().unwrap().ch_question_state.clone();
    let auth_token = AuthToken::get_or_create(&session).await?;
    let player_index = live_poll.lock().unwrap().get_player_index(&auth_token)?;

    let stream = WatchStream::new(updates)
        .map(move |state| match state {
            QuestionAreaState::None => sse::Event::default()
            .event("update")
            .data(""),
            QuestionAreaState::JoinCode(_) => sse::Event::default()
            .event("update")
            .data(html! {
                ."mt-20 mb-4 text-center text-slate-500" {
                    "Waiting for the host to start the poll."
                }
                ."flex justify-center" {
                    ."size-4" { (SvgIcon::Spinner.render()) }
                }
            }.into_string()),
            QuestionAreaState::Item { item_idx, is_last_question: _ } => {
                let mut live_poll = live_poll.lock().unwrap();
                let current_item = live_poll.get_current_item();

                sse::Event::default()
                .event("update")
                .data(
                    html! {
                        ."mb-2 flex gap-6 text-sm text-slate-500" {
                            "Question " (item_idx + 1)
                            @match &current_item.answers {
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
                            (current_item.question)
                        }
                        @match &current_item.answers {
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
                                (ft_answers.render_form(player_index, poll_id))
                            }
                        }
                        ."mt-36 text-sm text-slate-500 text-center" {
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

    let player_index = live_poll.get_player_index(&auth_token)?;
    let start_time = live_poll.get_current_item_start_time();

    let score =
        if let LiveAnswers::SingleChoice(mc_answers) = &mut live_poll.get_current_item().answers {
            mc_answers.submit_answer(player_index, form.answer_idx, start_time)?
        } else {
            return Err(AppError::BadRequest(
                "This is not a multiple choice item".to_string(),
            ));
        };

    if score != 0 {
        live_poll
            .get_current_item()
            .submit_score(player_index, score);

        let _ = live_poll.ch_players_updated_send.send(());
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
    pub free_text_answer: SmartString<Compact>,
}

pub async fn post_free_text_answer(
    Path(poll_id): Path<ShortID>,
    session: Session,
    Form(form_data): Form<PostFreeTextAnswerForm>,
) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let mut live_poll = live_poll.lock().unwrap();

    let player_index = live_poll.get_player_index(&auth_token)?;

    let response =
        if let LiveAnswers::FreeText(ft_answers) = &mut live_poll.get_current_item().answers {
            ft_answers.submit_answer(player_index, form_data.free_text_answer)?;
            ft_answers.render_form(player_index, poll_id)
        } else {
            return Err(AppError::BadRequest(
                "This is not a free text item".to_string(),
            ));
        };

    live_poll
        .ch_question_statistics_send
        .send_if_modified(|_stats| {
            return true;
        });

    return Ok(response.into_response());
}

fn render_poll_finished() -> Markup {
    html! {
        (render_header(html! {}))
        ."my-36 text-center text-slate-500" {
            "This poll has finished. Thank you for participating on svoote.com"
        }
    }
}
