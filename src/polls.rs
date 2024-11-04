use axum::{
    extract::{Multipart, Path},
    response::{IntoResponse, Response},
    Form, Json,
};
use maud::html;
use serde::{Deserialize, Serialize};
use smartstring::{Compact, SmartString};
use tower_sessions::Session;

use crate::{
    app_error::AppError,
    auth_token::AuthToken,
    config::{
        COLOR_PALETTE, MAX_FREE_TEXT_ANSWERS, POLL_MAX_ITEMS, POLL_MAX_MC_ANSWERS, POLL_MAX_STR_LEN,
    },
    host,
    html_page::{self, render_header},
    live_poll::{Answers, Item, LivePoll},
    live_poll_store::LIVE_POLL_STORE,
    static_file,
    svg_icons::SvgIcon,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistablePoll {
    pub items: Vec<Item>,
    pub version: usize,
    pub leaderboard_enabled: bool,
    pub question_time_limit_seconds: Option<usize>,
    pub allow_custom_names: bool,
}

impl PersistablePoll {
    pub fn new() -> Self {
        return PersistablePoll {
            items: Vec::new(),
            version: 1,
            leaderboard_enabled: false,
            question_time_limit_seconds: None,
            allow_custom_names: true,
        };
    }

    pub async fn from_session(session: &Session) -> Result<Self, AppError> {
        return Ok(session
            .get::<PersistablePoll>("poll_v1")
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .unwrap_or(PersistablePoll::new()));
    }

    pub async fn save_to_session(&self, session: &Session) -> Result<(), AppError> {
        return Ok(session
            .insert("poll_v1", self)
            .await
            .map_err(|e| AppError::DatabaseError(e))?);
    }
}

pub async fn post_start_poll(session: Session) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;

    let (poll_id, _live_poll) = match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, live_poll)) => (poll_id, live_poll),
        None => {
            let poll = PersistablePoll::from_session(&session).await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

            let (poll_id, live_poll) = LivePoll::orchestrate(poll.clone(), auth_token)?;

            session
                .insert("live_poll_id", poll_id)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;

            live_poll
                .lock()
                .unwrap()
                .ch_start_signal
                .take()
                .map(|signal| {
                    let _ = signal.send(());
                });

            (poll_id, live_poll)
        }
    };

    return host::render_live_host(poll_id).await;
}

pub async fn get_poll_page(session: Session) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;

    match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, _live_poll)) => return host::render_live_host(poll_id).await,
        None => {
            let poll = PersistablePoll::from_session(&session).await?;
            return render_edit_page(session, poll, 0).await;
        }
    };
}

async fn render_edit_page(
    session: Session,
    mut poll: PersistablePoll,
    active_item_index: usize,
) -> Result<Response, AppError> {
    if poll.items.len() == 0 {
        poll.items.push(Item {
            question: String::new(),
            answers: Answers::Untyped,
        });

        poll.save_to_session(&session).await?;
    }

    return Ok(html_page::render_html_page("Svoote", html! {
        #pollEditingArea ."mb-16" {
            (render_header(html! {
                ."flex items-center gap-3" {
                    ."text-sm text-slate-500" {
                        "Start poll"
                    }
                    button
                        #start-poll-btn
                        hx-post="/"
                        hx-select="main"
                        hx-target="main"
                        hx-swap="outerHTML"
                        disabled[poll.items.len() == 1 && matches!(poll.items[0].answers, Answers::Untyped)]
                        ."relative group size-12 text-slate-100 bg-cyan-600 rounded-full hover:bg-cyan-800 disabled:bg-slate-300"
                    {
                        ."group-[.htmx-request]:opacity-0 flex justify-center" { ."translate-x-[0.1rem] size-6" { (SvgIcon::Play.render()) } }
                        ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                            ."size-4" { (SvgIcon::Spinner.render()) }
                        }
                    }
                }
            }))
            div x-data="poll" {
                div ."mb-4 grid grid-cols-3" {
                    div x-data="{ open: false }" ."relative" {
                        button "@click"="open = !open" ."size-6 text-slate-400 hover:text-slate-600" { (SvgIcon::Settings.render()) }
                        div x-show="open" "@click.outside"="open = false" ."absolute left-0 top-8 w-64 h-fit z-10 p-4 text-left bg-white border rounded-lg shadow-lg" {
                            ."mb-3 text-xl font-semibold text-slate-700" { "Options" }
                            label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                input type="checkbox" x-model="poll.leaderboardEnabled" ."size-4 accent-indigo-500";
                                "Leaderboard"
                            }
                            ."ml-6 mb-3 text-slate-400 text-sm" { "Participants will receive points for submitting the correct answer. Faster responses get more points." }
                            label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                input type="checkbox" x-model="poll.allowCustomNames" ."size-4 accent-indigo-500";
                                "Custom names"
                            }
                            ."ml-6 mb-3 text-slate-400 text-sm" { "Allow participants to set a custom name." }
                            a download="poll.json" ":href"="'data:application/json;charset=utf-8,' + JSON.stringify(poll)" ."mb-3 flex gap-2 items-center text-slate-600 font-semibold" {
                                ."size-4" { (SvgIcon::Download.render()) }
                                "Save poll (.json)"
                            }
                            button ."flex gap-2 items-center text-slate-600 font-semibold" {
                                ."size-4" { (SvgIcon::UploadCloud.render()) }
                                "Import poll (.json)"
                            }
                        }
                    }
                }
                div ."flex gap-6 overflow-hidden" {
                    button "@click"="poll.activeSlide -= 1; save();" ":disabled"="poll.activeSlide == 0" ."z-10 relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:bg-slate-300" {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4 translate-x-[-0.05rem]" { (SvgIcon::ChevronLeft.render()) }
                        }
                    }
                    div ."flex-1 relative h-[36rem]" {
                        template x-for="(slide, slide_index) in poll.slides" {
                            div ."absolute inset-0 size-full px-12 py-8 border rounded transition duration-500 ease-out" ":class"="slide_index == poll.activeSlide && 'shadow-xl'" ":style"="'transform: translateX(' + ((slide_index - poll.activeSlide) * 101) + '%)'" {
                                input type="text" x-model="slide.question" "@input"="save"
                                    "@keyup.enter"="let e = document.getElementById('s-' + slide_index + '-mc-answer-0'); if (e !== null) e.focus(); else document.getElementById('add-mc-answer-' + slide_index).click();"
                                    placeholder="Question"
                                    ."w-full mb-6 text-xl text-slate-700 outline-none";
                                template x-if="slide.type == 'undefined'" {
                                    div {
                                        ."mb-2 text-slate-500 tracking-tight text-center" {
                                            "Choose item type:"
                                        }
                                        ."flex justify-center gap-4" {
                                            button "@click"="slide.type = 'mc'" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                                                "Multiple choice"
                                            }
                                            button "@click"="slide.type = 'ft'" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                                "Free text"
                                            }
                                        }
                                    }
                                }
                                template x-if="slide.type == 'mc'" {
                                    div {
                                        template x-for="(answer, answer_index) in slide.mcAnswers" {
                                            div ."mb-4 flex items-center gap-3" {
                                                div x-text="incrementChar('A', answer_index)" ."text-slate-500" {}
                                                input type="text"
                                                    x-model="answer.text"
                                                    "@input"="save()"
                                                    "@keyup.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                    ":id"="(answer_index == 0) && 's-' + slide_index + '-mc-answer-0'" ":class"="answer.isCorrect ? 'ring-[3px] ring-green-600' : 'ring-2 ring-slate-400 focus:ring-slate-500'"
                                                    ."w-full px-3 py-2 text-slate-700 rounded-lg outline-none";
                                                button "@click"="answer.isCorrect = !answer.isCorrect; save()" ":class"="answer.isCorrect ? 'text-green-600' : 'text-slate-300 hover:text-green-600'" ."size-6" { (SvgIcon::CheckSquare.render()) }
                                                button "@click"="slide.mcAnswers.splice(answer_index, 1); save();" ."size-6 text-slate-300 hover:text-slate-500" { (SvgIcon::Trash2.render()) }
                                            }
                                        }
                                        button
                                            "@click"={"if (slide.mcAnswers.length < " (POLL_MAX_MC_ANSWERS) ") { slide.mcAnswers.push({ text: '', isCorrect: false }); save(); $nextTick(() => $el.previousSibling.children[1].focus()); }" }
                                            ":class"={ "(slide.mcAnswers.length >= " (POLL_MAX_MC_ANSWERS) ") && 'hidden'" }
                                            ."ml-6 text-slate-700 underline"
                                            ":id"="'add-mc-answer-' + slide_index"
                                        {
                                            "Add answer"
                                        }
                                    }
                                }
                                template x-if="slide.type == 'ft'" {
                                    div {
                                        ."pl-2 flex gap-2 items-center text-slate-500" {
                                            ."size-4 shrink-0" { (SvgIcon::Edit3.render()) }
                                            "Free text: Participants can submit their own answer."
                                        }
                                    }
                                }
                            }
                        }
                    }
                    template x-if="poll.activeSlide + 1 < poll.slides.length" {
                        button "@click"="poll.activeSlide += 1; save();" ."relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700" {
                            ."absolute inset-0 size-full flex items-center justify-center" {
                                ."size-4" { (SvgIcon::ChevronRight.render()) }
                            }
                        }
                    }
                    template x-if="poll.activeSlide + 1 == poll.slides.length" {
                        button "@click"="poll.slides.push(createSlide()); save(); $nextTick(() => poll.activeSlide += 1);" ."relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700" {
                            ."absolute inset-0 size-full flex items-center justify-center" {
                                ."size-4" { (SvgIcon::Plus.render()) }
                            }
                        }
                    }
                }
                div ."flex justify-center" {
                    button "@click"="poll.slides.splice(poll.activeSlide, 1); if (poll.activeSlide == poll.slides.length) poll.activeSlide -= 1; save();" ":disabled"="poll.slides.length == 1" { "Delete slide" }
                }
            }
        }
        script src=(static_file::get_path("alpine.js")) {}
    }, true).into_response());
}

pub async fn post_add_item(session: Session) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;

    if poll.items.len() > POLL_MAX_ITEMS {
        return Err(AppError::BadRequest(
            "Poll can't contain more than 32 items".to_string(),
        ));
    }

    poll.items.push(Item {
        question: String::new(),
        answers: Answers::Untyped,
    });

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

#[derive(Deserialize)]
pub struct PutQuestionText {
    pub question: String,
}

pub async fn put_question_text(
    Path(slide_index): Path<usize>,
    session: Session,
    Form(form_data): Form<PutQuestionText>,
) -> Result<Response, AppError> {
    if form_data.question.chars().count() > 2048 {
        return Err(AppError::BadRequest(
            "Question text length limit of 2048 chars exceeded".to_string(),
        ));
    }

    let mut poll = PersistablePoll::from_session(&session).await?;

    match poll.items.get_mut(slide_index) {
        Some(item) => {
            item.question = form_data.question;
        }
        None => {
            return Err(AppError::BadRequest(
                "Item index out of bounds.".to_string(),
            ))
        }
    }

    poll.save_to_session(&session).await?;

    return Ok("Answer updated".into_response());
}

#[derive(Deserialize)]
pub struct PutMCAnswerForm {
    pub answer_text: String,
}

pub async fn put_mc_answer_text(
    Path((slide_index, answer_idx)): Path<(usize, usize)>,
    session: Session,
    Form(form_data): Form<PutMCAnswerForm>,
) -> Result<Response, AppError> {
    if form_data.answer_text.chars().count() > 2048 {
        return Err(AppError::BadRequest(
            "Answer text exceeds upper limit of 2048 characters".to_string(),
        ));
    }

    let mut poll = PersistablePoll::from_session(&session).await?;

    match &mut poll
        .items
        .get_mut(slide_index)
        .ok_or(AppError::BadRequest(
            "Item index out of bounds.".to_string(),
        ))?
        .answers
    {
        Answers::SingleChoice(answers) => {
            answers
                .get_mut(answer_idx)
                .ok_or(AppError::BadRequest(
                    "Answer index out of bounds.".to_string(),
                ))?
                .0 = form_data.answer_text;
        }
        _ => {
            return Err(AppError::BadRequest(
                "This is not a single choice item".to_string(),
            ));
        }
    }

    poll.save_to_session(&session).await?;

    Ok("Answer updated".into_response())
}

pub async fn put_mc_toggle_correct(
    Path((slide_index, answer_idx)): Path<(usize, usize)>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;

    match &mut poll
        .items
        .get_mut(slide_index)
        .ok_or(AppError::BadRequest(
            "Item index out of bounds.".to_string(),
        ))?
        .answers
    {
        Answers::SingleChoice(answers) => {
            answers
                .get_mut(answer_idx)
                .ok_or(AppError::BadRequest(
                    "Answer index out of bounds.".to_string(),
                ))?
                .1 ^= true;
        }
        _ => {
            return Err(AppError::BadRequest(
                "This is not a single choice item".to_string(),
            ));
        }
    }

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

pub async fn post_item_type(
    session: Session,
    Path((slide_index, item_type_descriptor)): Path<(usize, SmartString<Compact>)>,
) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;

    match poll.items.get_mut(slide_index) {
        Some(item) => match item_type_descriptor.as_str() {
            "single_choice" => item.answers = Answers::SingleChoice(Vec::new()),
            "free_text" => item.answers = Answers::FreeText(MAX_FREE_TEXT_ANSWERS, Vec::new()),
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Invalid item type: {}",
                    item_type_descriptor
                )));
            }
        },
        None => {
            return Err(AppError::BadRequest(
                "slide_index out of bounds".to_string(),
            ))
        }
    }

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

pub async fn delete_item(
    Path(slide_index): Path<usize>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;

    if slide_index >= poll.items.len() {
        return Err(AppError::BadRequest("Item index out of bounds".to_string()));
    }

    poll.items.remove(slide_index);

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

pub async fn get_poll_json(session: Session) -> Result<Json<PersistablePoll>, AppError> {
    return Ok(Json(PersistablePoll::from_session(&session).await?));
}

pub async fn post_poll_json(
    session: Session,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let uploaded_data = loop {
        if let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| AppError::BadRequest("Error uploading file.".to_string()))?
        {
            if field.name().is_some_and(|name| name == "poll_file") {
                break field
                    .bytes()
                    .await
                    .map_err(|_| AppError::BadRequest("Error uploading file.".to_string()))?;
            }
        } else {
            return Err(AppError::BadRequest("Missing poll_file field".to_string()));
        }
    };

    let uploaded_data = String::from_utf8(uploaded_data.to_vec())
        .map_err(|e| AppError::BadRequest(format!("UTF-8 error: {}", e)))?;

    let poll: PersistablePoll =
        serde_json::from_str(&uploaded_data).map_err(|e| AppError::BadRequest(e.to_string()))?;

    if poll.items.len() > POLL_MAX_ITEMS {
        return Err(AppError::BadRequest(
            "A poll must not contain more than 32 items.".to_string(),
        ));
    }

    for item in &poll.items {
        if item.question.len() > POLL_MAX_STR_LEN {
            return Err(AppError::BadRequest("MAX_STR_LEN reached".to_string()));
        }

        match &item.answers {
            Answers::SingleChoice(mc_answer) => {
                for (answer_text, _) in mc_answer {
                    if answer_text.len() > POLL_MAX_STR_LEN {
                        return Err(AppError::BadRequest("MAX_STR_LEN reached".to_string()));
                    }
                }
            }
            _ => {}
        }
    }

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

#[derive(Deserialize)]
pub struct EnableLeaderboardParams {
    pub enable_leaderboard: Option<SmartString<Compact>>,
}

pub async fn post_enable_leaderboard(
    session: Session,
    Form(params): Form<EnableLeaderboardParams>,
) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;
    poll.leaderboard_enabled = params.enable_leaderboard.is_some();
    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

#[derive(Deserialize)]
pub struct AllowCustomPlayerNamesParams {
    pub allow_custom_names: Option<SmartString<Compact>>,
}

pub async fn post_allow_custom_player_names(
    session: Session,
    Form(params): Form<AllowCustomPlayerNamesParams>,
) -> Result<Response, AppError> {
    let mut poll = PersistablePoll::from_session(&session).await?;
    poll.allow_custom_names = params.allow_custom_names.is_some();
    poll.save_to_session(&session).await?;

    return Ok("Updated custom player names setting".into_response());
}
