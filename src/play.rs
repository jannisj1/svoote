use crate::{
    app_error::AppError,
    auth_token::AuthToken,
    config::LIVE_POLL_PARTICIPANT_LIMIT,
    html_page::{self, render_header},
    live_poll::QuestionAreaState,
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    slide::SlideType,
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
    ("Rascal", include_str!("static/svgs/rascal_square.svg")),
    ("Chester", include_str!("static/svgs/chester_square.svg")),
    ("Coco", include_str!("static/svgs/coco_square.svg")),
    ("Bella", include_str!("static/svgs/bella_square.svg")),
    ("Gizmo", include_str!("static/svgs/gizmo_square.svg")),
    ("Kitty", include_str!("static/svgs/kitty_square.svg")),
    ("Daisy", include_str!("static/svgs/daisy_square.svg")),
    ("Angel", include_str!("static/svgs/angel_square.svg")),
    ("Bubba", include_str!("static/svgs/bubba_square.svg")),
    ("Boots", include_str!("static/svgs/boots_square.svg")),
    ("Patches", include_str!("static/svgs/patches_square.svg")),
    ("Simon", include_str!("static/svgs/simon_square.svg")),
    ("Sugar", include_str!("static/svgs/sugar_square.svg")),
    ("Gracie", include_str!("static/svgs/gracie_square.svg")),
    ("Princess", include_str!("static/svgs/princess_square.svg")),
    ("Dusty", include_str!("static/svgs/dusty_square.svg")),
    ("Luna", include_str!("static/svgs/luna_square.svg")),
    ("Baby", include_str!("static/svgs/baby_square.svg")),
    ("Milo", include_str!("static/svgs/milo_square.svg")),
    ("Jasmine", include_str!("static/svgs/jasmine_square.svg")),
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

    pub fn get_custom_name<'a>(&'a self) -> &'a Option<SmartString<Compact>> {
        return &self.custom_name;
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
                        {
                            ."max-w-64 p-4" {
                                ."mb-2 flex justify-end" {
                                    button
                                        onclick="document.getElementById('participant-dialog').close()"
                                        ."size-5 text-red-500"
                                    { (SvgIcon::X.render()) }
                                }
                                form
                                    onsubmit="submitParticipantNameDialog(event)"
                                    ."flex flex-col"
                                {
                                    label
                                        for="input-txt-participant-modal-name"
                                        ."text-slate-500"
                                    { "Name" }
                                    input type="text"
                                        ."mb-4 px-4 py-1.5 flex-1 text-slate-700 bg-slate-100 rounded-lg"
                                        #input-txt-participant-modal-name
                                        name="player_name"
                                        maxlength="32"
                                        placeholder=(player.get_generated_name())
                                        value=(player.get_custom_name().as_ref().unwrap_or(&SmartString::new()));
                                    ."mb-1 text-slate-500" { "Avatar" }
                                    ."flex flex-wrap gap-4" {
                                        @for avatar in AVATARS {
                                            ."size-8" { (PreEscaped(avatar.1)) }
                                        }
                                    }
                                    ."flex justify-end" {
                                        button ."bg-slate-600 p-4 rounded" { "Submit" }
                                    }
                                }
                            }
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
            QuestionAreaState::None => sse::Event::default().event("update").data(""),
            QuestionAreaState::Item(slide_idx) => sse::Event::default().event("update").data(
                live_poll
                    .lock()
                    .unwrap()
                    .get_current_item()
                    .render_participant_view(poll_id, slide_idx, player_index)
                    .into_string(),
            ),
            QuestionAreaState::PollFinished => sse::Event::default()
                .event("update")
                .data(render_poll_finished().into_string()),
            QuestionAreaState::CloseSSE => sse::Event::default().event("close").data(""),
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
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
        if let SlideType::SingleChoice(mc_answers) = &mut live_poll.get_current_item().slide_type {
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
        ."text-slate-700" { "Your answer has been submitted." }
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
        if let SlideType::FreeText(ft_answers) = &mut live_poll.get_current_item().slide_type {
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
