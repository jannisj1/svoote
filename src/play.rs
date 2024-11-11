use crate::{
    app_error::AppError,
    config::{CUSTOM_PLAYER_NAME_LENGTH_LIMIT, LIVE_POLL_PARTICIPANT_LIMIT},
    html_page::{self, render_header},
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
};
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use maud::{html, PreEscaped};
use serde::Deserialize;
use smartstring::{Compact, SmartString};
use std::fmt::Write;

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

    pub fn set_name(&mut self, new_name: SmartString<Compact>) -> Result<(), AppError> {
        let new_name = SmartString::from(new_name.trim());

        if new_name.len() > CUSTOM_PLAYER_NAME_LENGTH_LIMIT {
            return Err(AppError::BadRequest(format!(
                "Name longer than custom name length limit ({})",
                CUSTOM_PLAYER_NAME_LENGTH_LIMIT
            )));
        }

        if new_name.is_empty() {
            self.custom_name = None;
        } else {
            self.custom_name = Some(new_name);
        }

        return Ok(());
    }

    pub fn get_generated_name<'a>(&'a self) -> &'a SmartString<Compact> {
        return &self.generated_name;
    }

    pub fn get_custom_name<'a>(&'a self) -> &'a Option<SmartString<Compact>> {
        return &self.custom_name;
    }

    pub fn get_avatar_index(&self) -> usize {
        return self.avatar_index;
    }

    pub fn set_avatar_index(&mut self, new_index: usize) -> Result<(), AppError> {
        if new_index >= AVATARS.len() {
            return Err(AppError::BadRequest(
                "Avatar index out of bounds".to_string(),
            ));
        }

        self.avatar_index = new_index;

        return Ok(());
    }

    pub fn get_avatar_svg(&self) -> PreEscaped<&'static str> {
        return PreEscaped(AVATARS[self.avatar_index].1);
    }
}

#[derive(Deserialize)]
pub struct PlayPageParams {
    pub c: Option<ShortID>,
}

pub async fn get_play_page(
    Query(params): Query<PlayPageParams>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);
    let live_poll = params
        .c
        .map(|poll_id| LIVE_POLL_STORE.get(poll_id))
        .flatten();

    if live_poll.is_none() {
        let html = html_page::render_html_page(
            "Svoote",
            html! {
               (render_header(html! {}))
               div { "TODO" }
            },
        );

        return Ok((cookies, html).into_response());
    }

    let live_poll = live_poll.unwrap();
    let mut live_poll = live_poll.lock().unwrap();

    let html = html_page::render_html_page(
        "Svoote",
        match live_poll.get_or_create_player(&session_id) {
            Some(player_index) => {
                let _player = live_poll.get_player(player_index);
                html! {
                    /*(render_header(html! {
                        (render_name_avatar_button(live_poll.leaderboard_enabled, player.get_name(), player.get_avatar_svg()))
                        @if live_poll.leaderboard_enabled {
                            dialog #participant-dialog
                            {
                                ."max-w-64 p-4 bg-slate-700 rounded-lg" {
                                    ."mb-2 flex justify-end" {
                                        button
                                            onclick="document.getElementById('participant-dialog').close()"
                                            ."size-5 text-red-500"
                                        { (SvgIcon::X.render()) }
                                    }
                                    form
                                        hx-post={ "/name_avatar/" (params.c) }
                                        hx-target="#name-avatar-button"
                                        hx-swap="outerHTML"
                                        "hx-on::after-request"="document.getElementById('participant-dialog').close()"
                                            //onsubmit="submitParticipantNameDialog(event)"
                                        ."flex flex-col"
                                    {
                                        label
                                            for="input-txt-participant-modal-name"
                                            ."mb-1 text-slate-300"
                                        { "Name" }
                                        input type="text"
                                            ."mb-1 px-4 py-1.5 flex-1 text-slate-700 bg-slate-100 rounded-lg"
                                            #input-txt-participant-modal-name
                                            name="name"
                                            maxlength=(CUSTOM_PLAYER_NAME_LENGTH_LIMIT)
                                            placeholder=(player.get_generated_name())
                                            value=(player.get_custom_name().as_ref().unwrap_or(&SmartString::new()))
                                            disabled[!live_poll.allow_custom_player_names];
                                        @if !live_poll.allow_custom_player_names {
                                            ."text-slate-300 text-sm" { "The host has turned off custom names." }
                                        }
                                        ."mb-4" {}
                                        ."mb-2 text-slate-300" { "Avatar" }
                                        ."mb-6 flex justify-around flex-wrap gap-4" {
                                            @for (i, avatar) in AVATARS.iter().enumerate() {
                                                label {
                                                    input type="radio" name="avatar" value=(i) checked[player.get_avatar_index() == i] ."peer hidden";
                                                    ."size-9 p-0.5 ring-slate-100 rounded peer-checked:ring-2" { (PreEscaped(avatar.1)) }
                                                }
                                            }
                                        }
                                        ."flex justify-end" {
                                            button type="submit" ."bg-slate-100 px-4 py-2 text-slate-800 rounded hover:bg-slate-300" { "Submit" }
                                        }
                                    }
                                }
                            }
                        }
                    }))*/
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
    );

    return Ok((cookies, html).into_response());
}

/*pub async fn get_sse_play(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let updates = live_poll.lock().unwrap().ch_question_state.clone();
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    let player_index = live_poll.lock().unwrap().get_player_index(&session_id)?;

    let stream = WatchStream::new(updates)
        .map(move |state| match state {
            QuestionAreaState::Empty => sse::Event::default().event("update").data(""),
            QuestionAreaState::Slide(slide_idx) => sse::Event::default().event("update").data(
                live_poll
                    .lock()
                    .unwrap()
                    .get_current_slide()
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
}*/
/*
#[derive(Deserialize)]
pub struct PostMCAnswerForm {
    pub answer_idx: usize,
}

pub async fn post_mc_answer(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
    Form(form): Form<PostMCAnswerForm>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    let player_index = live_poll.get_player_index(&session_id)?;
    let start_time = live_poll.get_current_slide_start_time();

    let score = if let SlideType::MultipleChoice(mc_answers) =
        &mut live_poll.get_current_slide().slide_type
    {
        mc_answers.submit_answer(player_index, form.answer_idx, start_time)?
    } else {
        return Err(AppError::BadRequest(
            "This is not a multiple choice item".to_string(),
        ));
    };

    if score != 0 {
        live_poll
            .get_current_slide()
            .submit_score(player_index, score);

        //let _ = live_poll.ch_players_updated_send.send(());
    }

    /*live_poll
    .ch_question_statistics_send
    .send_if_modified(|_stats| true);*/

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
    cookies: CookieJar,
    Form(form_data): Form<PostFreeTextAnswerForm>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    let player_index = live_poll.get_player_index(&session_id)?;

    let response =
        if let SlideType::FreeText(ft_answers) = &mut live_poll.get_current_slide().slide_type {
            ft_answers.submit_answer(player_index, form_data.free_text_answer)?;
            ft_answers.render_participant_form(player_index, poll_id)
        } else {
            return Err(AppError::BadRequest(
                "This is not a free text item".to_string(),
            ));
        };

    /*live_poll
    .ch_question_statistics_send
    .send_if_modified(|_stats| {
        return true;
    });*/

    return Ok(response.into_response());
}

pub fn render_poll_finished() -> Markup {
    html! {
        ."mx-auto mt-12 mb-6 w-24 md:w-32" { (Illustrations::InLove.render()) }
        ."text-center text-sm text-slate-500" {
            "This poll is finished." br;
            "Thank you for using svoote.com"
        }
    }
}*/
/*
fn render_name_avatar_button(
    leaderboard_enabled: bool,
    player_name: &SmartString<Compact>,
    avatar_svg: PreEscaped<&str>,
) -> Markup {
    return html! {
        button
            #name-avatar-button
            ."px-3 py-1 flex items-center gap-2.5 rounded-lg bg-slate-100 hover:bg-slate-200 disabled:bg-slate-100"
            onclick="document.getElementById('participant-dialog').showModal()"
            disabled[!leaderboard_enabled]
        {
            ."text-slate-600" {
                @if leaderboard_enabled {
                    (player_name)
                } @else {
                    "Anonymous"
                }
            }
            ."size-6 text-slate-500" {
                @if leaderboard_enabled {
                    (avatar_svg)
                } @else {
                    (SvgIcon::User.render())
                }
            }
        }
    };
}*/

/*#[derive(Deserialize)]
pub struct NameAvatarParams {
    pub name: Option<SmartString<Compact>>,
    pub avatar: usize,
}

pub async fn post_name_avatar(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
    Form(params): Form<NameAvatarParams>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    let player_index = live_poll.get_player_index(&session_id)?;

    let leaderboard_enabled = live_poll.leaderboard_enabled;
    let allow_custom_player_names = live_poll.allow_custom_player_names;

    let (player_name, avatar_svg) = {
        let player = live_poll.get_player_mut(player_index);

        if allow_custom_player_names {
            player.set_name(params.name.unwrap_or(SmartString::new()))?;
        }

        player.set_avatar_index(params.avatar)?;

        (player.get_name(), player.get_avatar_svg())
    };

    return Ok(
        render_name_avatar_button(leaderboard_enabled, player_name, avatar_svg).into_response(),
    );
}*/
