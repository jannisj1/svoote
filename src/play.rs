use crate::{
    app_error::AppError,
    config::{FREE_TEXT_MAX_CHAR_LENGTH, LIVE_POLL_PARTICIPANT_LIMIT, POLL_MAX_MC_ANSWERS},
    html_page::{self, render_header},
    live_poll::LivePoll,
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    select_language, session_id,
    slide::{Slide, SlideType, WordCloudTerm},
    start_page::render_join_form,
    wsmessage::WSMessage,
};
use arrayvec::ArrayVec;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, WebSocketUpgrade,
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
    Form, Json,
};
use axum_extra::extract::CookieJar;

use maud::html;
use serde::Deserialize;
use serde_json::{json, Value};
use smartstring::{Compact, SmartString};
use std::{
    collections::HashMap,
    fmt::Write,
    sync::{Arc, Mutex},
};
use tokio::select;

pub async fn get_poll_exists(Path(poll_id): Path<ShortID>) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id);

    if live_poll.is_some() {
        return Ok("true".into_response());
    } else {
        return Ok("false".into_response());
    }
}

#[derive(Deserialize)]
pub struct PlayPageParams {
    pub c: Option<SmartString<Compact>>,
}

pub async fn get_play_page(
    Query(params): Query<PlayPageParams>,
    cookies: CookieJar,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    let poll_id_str = params.c.clone().unwrap_or(SmartString::new());
    let poll_id: Option<ShortID> = params
        .c
        .clone()
        .map(|poll_id| poll_id.parse().ok())
        .flatten();

    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);
    let live_poll = poll_id
        .map(|poll_id| LIVE_POLL_STORE.get(poll_id))
        .flatten();

    if live_poll.is_none() {
        let html = html_page::render_html_page(
            "Svoote",
            &l,
            html! {
                (render_header(html! {}))
                (render_join_form(&l))
                div ."my-24 mx-6 sm:mx-14" {
                    p ."mb-2 text-center text-sm text-slate-500" {
                        @if params.c.is_some() {
                            (t!("poll_finished", locale=l))
                        } @else {
                            (t!("enter_code_above", locale=l))
                        }
                    }
                    ."flex justify-center" {
                        a ."text-sm text-cyan-600 underline"
                            href="/" { (t!("goto_start_page", locale=l)) " →" }
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
        &l,
        match live_poll.get_or_create_player(&session_id) {
            Some(player_index) => {
                let _player = live_poll.get_player(player_index);
                html! {
                    script { "document.code = " (poll_id.unwrap_or(0)) ";" }
                    (render_header(html! {}))
                    div x-data="participant" ."mt-12 mb-20 mx-6 sm:mx-14" {
                        div ."w-full max-w-96 mx-auto" {
                            template x-if="currentSlide.slideType == 'null'" { div {} }
                            template x-if="currentSlide.slideType == 'mc'" {
                                div {
                                    h1 x-init="$el.innerText = currentSlide.question" x-effect="$el.innerText = currentSlide.question" ."mb-4 text-lg text-slate-700 font-medium" {}
                                    template x-for="(answer, answerIndex) in currentSlide.answers" {
                                        label ."w-full mb-4 px-3 py-1.5 flex gap-2 items-center ring-2 ring-slate-500 has-checked:ring-4 has-checked:ring-cyan-600 rounded-lg transition" {
                                            input ":type"="currentSlide.allowMultipleMCAnswers ? 'checkbox' : 'radio'" x-model="currentSlide.selectedAnswer" ":disabled"="currentSlide.submitted" ":value"="answerIndex" ."accent-cyan-600";
                                            div ."text-slate-700 font-medium" x-text="answer.text" {}
                                        }
                                    }
                                    div ."relative mt-7 h-10" {
                                        button x-show="!currentSlide.submitted"
                                            ":disabled"="(currentSlide.allowMultipleMCAnswers && currentSlide.selectedAnswer.length === 0) || (!currentSlide.allowMultipleMCAnswers && currentSlide.selectedAnswer === '')"
                                            "@click"={ "submitMCAnswer(" (poll_id_str) ")" }
                                            ."absolute size-full inset-0 flex items-center justify-center text-white font-bold bg-cyan-600 rounded-full cursor-pointer disabled:cursor-default hover:bg-cyan-700 disabled:bg-slate-300"
                                            { (t!("submit", locale=l)) }
                                        div x-show="currentSlide.submitted"
                                            ."absolute size-full inset-0 flex items-center justify-center text-slate-500 text-sm"
                                            { (t!("answer_submitted", locale=l)) }
                                    }
                                }
                            }
                            template x-if="currentSlide.slideType == 'ft'" {
                                div {
                                    h1 x-init="$el.innerText = currentSlide.question" x-effect="$el.innerText = currentSlide.question" ."mb-5 text-lg text-slate-700 font-medium" {}
                                    input type="text"
                                        x-model="currentSlide.selectedAnswer"
                                        "@keyup.enter"="$refs.ftSubmitButton.click()"
                                        ":disabled"="currentSlide.submitted"
                                        placeholder=(t!("answer", locale=l))
                                        ."w-full px-4 py-1.5 text-lg text-slate-700 font-medium ring-2 ring-slate-500 rounded-lg outline-hidden focus:ring-4 focus:ring-cyan-600 transition";
                                    div ."relative mt-5 h-10" {
                                    button x-show="!currentSlide.submitted"
                                        x-ref="ftSubmitButton"
                                        ":disabled"="currentSlide.selectedAnswer === ''"
                                        "@click"={ "submitFTAnswer(" (poll_id_str) ")" }
                                        ."absolute size-full inset-0 flex items-center justify-center text-white font-bold bg-cyan-600 rounded-full cursor-pointer disabled:cursor-default hover:bg-cyan-700 disabled:bg-slate-300"
                                        { (t!("submit", locale=l)) }
                                    div x-show="currentSlide.submitted"
                                        ."absolute size-full inset-0 flex items-center justify-center text-slate-500 text-sm"
                                        { (t!("answer_submitted", locale=l)) }
                                    }
                                }
                            }
                            hr ."mt-12 mb-5";
                            p ."mb-3 text-xs text-center text-slate-500" { (t!("your_reaction", locale=l)) }
                            div ."flex justify-center gap-4" {
                                @for emoji in [("heart", "❤️"), ("thumbsUp", "👍"), ("thumbsDown", "👎"), ("smileyFace", "😀"), ("sadFace", "🙁")] {
                                    button "@click"={ "submitEmoji(" (poll_id_str) ", '" (emoji.0) "')" }
                                        ."relative size-10 rounded-full border shadow-xs cursor-pointer hover:bg-slate-100 disabled:pointer-events-none transition"
                                        ":class"={ "currentSlide.emoji == '" (emoji.0) "' ? 'disabled:scale-[1.2] disabled:bg-cyan-600 disabled:bg-opacity-70' : 'disabled:shadow-none disabled:opacity-50'" }
                                        ":disabled"="currentSlide.emoji != null"
                                        { div ."absolute left-1/2 top-1/2 translate-x-[-50%] translate-y-[-50%] text-base" { (emoji.1) } }
                                }
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
const _AVATARS: &[(&'static str, &'static str)] = &[
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
    //generated_name: SmartString<Compact>,
    //custom_name: Option<SmartString<Compact>>,
    //avatar_index: usize,
}

impl Player {
    pub fn new(_player_index: usize) -> Self {
        return Player {};
        /*let avatar_index = player_index % AVATARS.len();
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
        };*/
    }

    /*
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
    }*/
}

#[derive(Deserialize)]
pub struct PostMCAnswerForm {
    pub slide_index: usize,
    pub answer_indices: ArrayVec<u8, POLL_MAX_MC_ANSWERS>,
}

pub async fn post_mc_answer(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
    Json(form): Json<PostMCAnswerForm>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    if form.slide_index >= live_poll.slides.len() {
        return Err(AppError::BadRequest(
            "slide_index out of bounds".to_string(),
        ));
    }

    let player_index = live_poll.get_player_index(&session_id)?;
    let start_time = live_poll.get_current_slide_start_time();

    let score = if let SlideType::MultipleChoice(mc_answers) =
        &mut live_poll.slides[form.slide_index].slide_type
    {
        mc_answers.submit_answer(player_index, form.answer_indices, start_time)?
    } else {
        return Err(AppError::BadRequest(
            "This is not a multiple choice item".to_string(),
        ));
    };

    if score > 0 {
        live_poll
            .get_current_slide()
            .submit_score(player_index, score);
    }

    let _ = live_poll
        .stats_change_notification_channel_sender
        .send(form.slide_index);

    return Ok(html! {}.into_response());
}

#[derive(Deserialize)]
pub struct PostFreeTextAnswerForm {
    pub answer: SmartString<Compact>,
    pub slide_index: usize,
}

pub async fn post_ft_answer(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
    Form(form): Form<PostFreeTextAnswerForm>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    let player_index = live_poll.get_player_index(&session_id)?;
    if form.slide_index >= live_poll.slides.len() {
        return Err(AppError::BadRequest(
            "slide_index out of bounds".to_string(),
        ));
    }

    if let SlideType::FreeText(ft_answers) = &mut live_poll.slides[form.slide_index].slide_type {
        if ft_answers.player_answers[player_index].is_some() {
            return Err(AppError::BadRequest(
                "Already submitted an answer".to_string(),
            ));
        }

        let trimmed_answer = SmartString::from(form.answer.trim());

        let lowercase_answer = trimmed_answer
            .to_lowercase()
            .chars()
            .take(FREE_TEXT_MAX_CHAR_LENGTH)
            .collect::<SmartString<Compact>>();

        ft_answers.player_answers[player_index] = Some(form.answer);

        let term_index = ft_answers
            .word_cloud_terms
            .iter()
            .position(|term| term.lowercase_text == lowercase_answer);

        if let Some(term_index) = term_index {
            let term = &mut ft_answers.word_cloud_terms[term_index];
            term.count += 1;
            if term.count > ft_answers.max_term_count {
                ft_answers.max_term_count = term.count;
            }

            if let Some(spelling_count) = term.spellings.get_mut(&trimmed_answer) {
                *spelling_count += 1;
                if *spelling_count > term.highest_spelling_count {
                    term.highest_spelling_count = *spelling_count;
                    term.preferred_spelling = trimmed_answer;
                }
            } else {
                term.spellings.insert(trimmed_answer, 1);
            }
        } else {
            let mut spellings = HashMap::new();
            spellings.insert(trimmed_answer.clone(), 1);

            ft_answers.word_cloud_terms.push(WordCloudTerm {
                lowercase_text: lowercase_answer,
                count: 1,
                preferred_spelling: SmartString::from(trimmed_answer),
                spellings,
                highest_spelling_count: 1,
            });
        }
    } else {
        return Err(AppError::BadRequest(
            "This is not a free text item".to_string(),
        ));
    };

    let _ = live_poll
        .stats_change_notification_channel_sender
        .send(form.slide_index);

    return Ok("Answer submitted".into_response());
}

#[derive(Deserialize)]
pub struct PostEmojiForm {
    pub emoji: SmartString<Compact>,
    pub slide_index: usize,
}

pub async fn post_emoji(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
    Form(form): Form<PostEmojiForm>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let mut live_poll = live_poll.lock().unwrap();
    let player_index = live_poll.get_player_index(&session_id)?;
    if form.slide_index >= live_poll.slides.len() {
        return Err(AppError::BadRequest(
            "slide_index out of bounds".to_string(),
        ));
    }

    let slide = &mut live_poll.slides[form.slide_index];
    if let Some(emoji) = slide.player_emojis.get_mut(player_index) {
        if emoji.is_some() {
            return Err(AppError::BadRequest("Emoji already submitted".to_string()));
        }

        match form.emoji.as_str() {
            "heart" => slide.heart_emojis += 1,
            "thumbsUp" => slide.thumbs_up_emojis += 1,
            "thumbsDown" => slide.thumbs_down_emojis += 1,
            "smileyFace" => slide.smiley_face_emojis += 1,
            "sadFace" => slide.sad_face_emojis += 1,
            _ => return Err(AppError::BadRequest("Unknown emoji".to_string())),
        }

        *emoji = Some(form.emoji.clone());

        let _ = live_poll
            .emoji_channel_sender
            .send((form.slide_index, form.emoji));
    }

    return Ok("Emoji submitted".into_response());
}

pub async fn play_socket(
    ws: WebSocketUpgrade,
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    let player_index = live_poll.lock().unwrap().get_player_index(&session_id)?;

    return Ok(ws.on_upgrade(move |socket| handle_play_socket(socket, live_poll, player_index)));
}

async fn handle_play_socket(
    mut socket: WebSocket,
    live_poll: Arc<Mutex<LivePoll>>,
    player_index: usize,
) {
    let mut slide_index_change_receiver = {
        let live_poll = live_poll.lock().unwrap();

        live_poll
            .slide_change_notification_channel_receiver
            .resubscribe()
    };

    let msg = {
        let mut live_poll = live_poll.lock().unwrap();
        let current_slide_index = live_poll.current_slide_index;
        let slide = live_poll.get_current_slide();
        create_slide_ws_message(current_slide_index, slide, player_index).into()
    };
    let _ = socket.send(msg).await;

    loop {
        select! {
            msg = socket.recv() => {
                if let Some(Ok(msg)) = msg {
                    if let Some(_msg) = WSMessage::parse(msg) { }
                } else {
                    return;
                }
            }
            slide_index = slide_index_change_receiver.recv() => {
                if let Ok(slide_index) = slide_index {
                    let msg = {
                        let mut live_poll = live_poll.lock().unwrap();
                        let slide = live_poll.get_current_slide();
                        create_slide_ws_message(slide_index, slide, player_index).into()
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

fn create_slide_ws_message(slide_index: usize, slide: &Slide, player_index: usize) -> WSMessage {
    let emoji = match &slide.player_emojis[player_index] {
        Some(emoji) => json! { emoji },
        None => Value::Null,
    };

    let slide_json = match &slide.slide_type {
        SlideType::MultipleChoice(answers) => {
            let selected_answer = if answers.allow_multiple_answers {
                json! { answers.player_answers[player_index].as_ref().unwrap_or(&ArrayVec::new()) }
            } else {
                json! {
                answers.player_answers[player_index]
                    .as_ref()
                    .map(|answer_indices| {
                        let mut s = SmartString::<Compact>::new();
                        let _ = write!(&mut s, "{}", *answer_indices.get(0).unwrap_or(&0u8));
                        s
                    })
                    .unwrap_or(SmartString::new()) }
            };

            json!({
                "slideType": "mc",
                "question": slide.question,
                "answers": answers.answers.iter().map(|(answer_text, _is_correct)| json!({ "text": answer_text })).collect::<Vec<Value>>(),
                "submitted": answers.player_answers[player_index].is_some(),
                "selectedAnswer": selected_answer,
                "allowMultipleMCAnswers": answers.allow_multiple_answers,
                "emoji": emoji,
            })
        }
        SlideType::FreeText(answers) => {
            json!({
                "slideType": "ft",
                "question": slide.question,
                "selectedAnswer": answers.player_answers[player_index].as_ref().unwrap_or(&SmartString::new()),
                "submitted": answers.player_answers[player_index].is_some(),
                "emoji": emoji,
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
