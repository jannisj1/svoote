use crate::{
    app_error::AppError,
    config::{CUSTOM_PLAYER_NAME_LENGTH_LIMIT, LIVE_POLL_PARTICIPANT_LIMIT},
    html_page::{self, render_header},
    illustrations::Illustrations,
    live_poll::LivePoll,
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
    slide::{Slide, SlideType},
    svg_icons::SvgIcon,
    wsmessage::WSMessage,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use maud::{html, PreEscaped};
use serde::Deserialize;
use serde_json::{json, Value};
use smartstring::{Compact, SmartString};
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};
use tokio::select;

#[derive(Deserialize)]
pub struct PlayPageParams {
    pub c: Option<SmartString<Compact>>,
}

pub async fn get_play_page(
    Query(params): Query<PlayPageParams>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let poll_id_str = params.c.clone().unwrap_or(SmartString::new());
    let poll_id: Option<ShortID> = params.c.map(|poll_id| poll_id.parse().ok()).flatten();

    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);
    let live_poll = poll_id
        .map(|poll_id| LIVE_POLL_STORE.get(poll_id))
        .flatten();

    if live_poll.is_none() {
        let html = html_page::render_html_page(
            "Svoote",
            html! {
                div ."my-16 mx-4 sm:mx-14" {
                    form ."w-full max-w-64 mx-auto" {
                        div ."flex items-baseline justify-center gap-2 mb-8 text-3xl font-semibold tracking-tight" {
                            "Svoote" ."size-5 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
                        }
                        div ."mb-2 text-center text-sm text-slate-500" { "Enter the 4-digit code you see in front." }
                        input name="c" type="text" pattern="[0-9]*" inputmode="numeric" placeholder="Code" value=(poll_id_str)
                            ."w-full px-3 py-1.5 w-40 text-slate-700 text-lg font-medium border-2 border-slate-500 focus:border-slate-700 rounded-lg outline-none";
                        @if let Some(c) = poll_id { div ."mt-1 text-sm text-red-500" { "No poll with code " (c) " found." } }
                        button type="submit" ."w-full mt-6 py-1.5 text-center text-lg text-white font-bold bg-slate-700 hover:bg-slate-500 rounded-lg" { "Join" }
                    }
                    hr ."my-16 max-w-64 mx-auto border-slate-700";
                    div ."max-w-64 mx-auto" {
                        div ."mb-4" { (Illustrations::TeamCollaboration.render()) }
                        h1 ."mb-5 text-2xl text-center font-bold tracking-tight" { "Want to create your own polls?" }
                        a href="/" ."block w-fit mx-auto px-4 py-1 text-indigo-600 font-bold tracking-tight border rounded-full shadow hover:bg-slate-100" { "Start now →"}
                    }
                }
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
                    script { "document.code = " (poll_id.unwrap_or(0)) ";" }
                    div x-data="participant" {
                        div ."my-16 mx-4 sm:mx-14" {
                            div ."w-full max-w-80 mx-auto" {
                                div ."flex items-baseline justify-center gap-2 mb-12 text-3xl font-semibold tracking-tight" {
                                    "Svoote" ."size-5 translate-y-[0.1rem]" { (SvgIcon::Rss.render()) }
                                }
                                template x-if="currentSlide.slideType == 'null'" { div {} }
                                template x-if="currentSlide.slideType == 'mc'" {
                                    div x-data="{ selectedAnswer: '' }" {
                                        h1 x-text="currentSlide.question" ."mb-8 text-xl text-slate-800 font-medium tracking-tight leading-5" {}
                                        template x-for="(answer, answerIndex) in currentSlide.answers" {
                                            label ."w-full mb-5 px-4 py-2 flex gap-4 items-center ring-[3px] ring-slate-500 has-[:checked]:ring-4 has-[:checked]:ring-indigo-500 rounded-lg" {
                                                input type="radio" x-model="selectedAnswer" ":value"="answerIndex" ."accent-indigo-500 size-[1.2rem]";
                                                div ."text-slate-800 text-lg font-medium" x-text="answer.text" {}
                                            }
                                        }
                                        button "@click"="submitMCAnswer(selectedAnswer)" ."w-full mt-6 py-1.5 text-center text-lg text-white font-bold bg-slate-700 hover:bg-slate-500 rounded-lg" { "Submit" }
                                    }

                                }
                                template x-if="currentSlide.slideType == 'ft'" {
                                    div ."mb-2 text-center text-sm text-slate-500" { "Enter the 4-digit code you see in front." }
                                    input name="c" type="text" pattern="[0-9]*" inputmode="numeric" placeholder="Code" value=(poll_id_str)
                                        ."w-full px-3 py-1.5 w-40 text-slate-700 text-lg font-medium border-2 border-slate-500 focus:border-slate-700 rounded-lg outline-none";
                                    div { "Free text" }
                                }
                            }
                            //hr ."my-16 max-w-64 mx-auto border-slate-700";
                            div ."max-w-64 mx-auto" {
                                //a href="/" ."block w-fit mx-auto px-4 py-1 text-indigo-600 font-bold tracking-tight border rounded-full shadow hover:bg-slate-100" { "Start now →"}
                            }
                        }
                    }
                }
            }
            None => {
                html! {
                    (render_header(html!{}))
                    ."my-36 text-center text-slate-500" {
                        "The participant limit for this poll was reached (" (LIVE_POLL_PARTICIPANT_LIMIT) " participants)."
                    }
                }
            }
        },
    );

    return Ok((cookies, html).into_response());
}

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

pub async fn play_socket(
    ws: WebSocketUpgrade,
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    //session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    return Ok(ws.on_upgrade(|socket| handle_play_socket(socket, live_poll)));
}

async fn handle_play_socket(mut socket: WebSocket, live_poll: Arc<Mutex<LivePoll>>) {
    let (mut _stats_updated_receiver, mut slide_index_change_receiver) = {
        let live_poll = live_poll.lock().unwrap();

        (
            live_poll
                .stats_change_notification_channel_receiver
                .resubscribe(),
            live_poll
                .slide_change_notification_channel_receiver
                .resubscribe(),
        )
    };

    let msg = {
        let mut live_poll = live_poll.lock().unwrap();
        let current_slide_index = live_poll.current_slide_index;
        let slide = live_poll.get_current_slide();
        create_slide_ws_message(current_slide_index, slide).into()
    };
    let _ = socket.send(msg).await;

    loop {
        select! {
            msg = socket.recv() => {
                if let Some(Ok(msg)) = msg {
                    if let Some(msg) = WSMessage::parse(msg) {
                        match msg.cmd.as_ref() {
                            "submitMCAnswer" => {
                                let answer_index = msg.data["answerIndex"].as_u64().unwrap_or(0u64) as usize;
                                info!("Answer: {}", answer_index);
                                //let _ = slide_index_sender.send(slide_index).await;
                            }
                            _ => {}
                        }
                    }
                } else {
                    return;
                }
            }
            slide_index = slide_index_change_receiver.recv() => {
                if let Ok(slide_index) = slide_index {
                    let msg = {
                        let mut live_poll = live_poll.lock().unwrap();
                        let slide = live_poll.get_current_slide();
                        create_slide_ws_message(slide_index, slide).into()
                    };
                    let _  = socket.send(msg).await;
                } else {
                    return;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                if socket.send(Message::Ping(Vec::new())).await.is_err() {
                    return;
                }
            }
        };
    }
}

fn create_slide_ws_message(slide_index: usize, slide: &Slide) -> WSMessage {
    let slide_json = match &slide.slide_type {
        SlideType::MultipleChoice(answers) => {
            json!({
                "slideType": "mc",
                "question": slide.question,
                "answers": answers.answers.iter().map(|(answer_text, _is_correct)| json!({ "text": answer_text })).collect::<Vec<Value>>(),
            })
        }
        SlideType::FreeText(_answers) => {
            json!({
                "slideType": "ft",
                "question": slide.question,
            })
        }
        _ => {
            json!({
                "slideType": "empty",
            })
        }
    };

    return WSMessage {
        cmd: SmartString::from("updateSlide"),
        data: json!({
            "slideIndex": slide_index,
            "slide": slide_json,
        }),
    };
}
