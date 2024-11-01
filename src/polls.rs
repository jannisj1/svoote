use axum::{
    extract::{Multipart, Path},
    response::{IntoResponse, Response},
    Form, Json,
};
use maud::{html, Markup};
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
    svg_icons::SvgIcon,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct PollV1 {
    pub items: Vec<Item>,
    pub version: usize,
    pub leaderboard_enabled: bool,
    pub question_time_limit_seconds: Option<usize>,
    pub allow_custom_names: bool,
}

impl PollV1 {
    pub fn new() -> Self {
        return PollV1 {
            items: Vec::new(),
            version: 1,
            leaderboard_enabled: false,
            question_time_limit_seconds: None,
            allow_custom_names: true,
        };
    }

    pub async fn from_session(session: &Session) -> Result<Self, AppError> {
        return Ok(session
            .get::<PollV1>("poll_v1")
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .unwrap_or(PollV1::new()));
    }

    pub async fn save_to_session(&self, session: &Session) -> Result<(), AppError> {
        return Ok(session
            .insert("poll_v1", self)
            .await
            .map_err(|e| AppError::DatabaseError(e))?);
    }
}

pub async fn post_poll_page(session: Session) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;

    let (poll_id, _lq) = match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, lq)) => (poll_id, lq),
        None => {
            let poll = PollV1::from_session(&session).await?;

            let (poll_id, lq) = LivePoll::orchestrate(poll.clone(), auth_token)?;

            session
                .insert("live_poll_id", poll_id)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;

            lq.lock().unwrap().ch_start_signal.take().map(|signal| {
                let _ = signal.send(());
            });

            (poll_id, lq)
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
        Some((poll_id, _lq)) => return host::render_live_host(poll_id).await,
        None => {
            let poll = PollV1::from_session(&session).await?;
            return render_edit_page(session, poll).await;
        }
    };
}

async fn render_edit_page(session: Session, mut poll: PollV1) -> Result<Response, AppError> {
    if poll.items.len() == 0 {
        poll.items.push(Item {
            question: String::new(),
            answers: Answers::Untyped,
        });

        poll.save_to_session(&session).await?;
    }

    return Ok(html_page::render_html_page("Svoote", html! {
        #pollEditingArea."mb-16" {
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
            ."mb-4 text-xl font-semibold text-slate-700" {
                "Poll items"
            }
            ."mb-8 flex flex-col gap-8" #poll-items {
                @for (item_idx, item) in poll.items.iter().enumerate() {
                    ."flex items-center gap-3" {
                        div ."relative size-7 rounded-full bg-slate-600" {
                            ."absolute inset-0 size-full flex justify-center items-center text-slate-50 text-sm font-bold" { (item_idx + 1) }
                        }
                        ."flex-1 px-5 py-4 border shadow rounded" {
                            ."mb-4 flex items-center gap-4" {
                                input type="text"
                                    name="question"
                                    ."px-4 py-1.5 flex-1 text-slate-900 font-medium bg-slate-100 rounded-lg"
                                    hx-put={ "/poll/item/" (item_idx) "/text" }
                                    hx-trigger="input changed delay:300ms"
                                    "hx-on::before-request"="bindSavingIndicator();"
                                    "hx-on::after-request"="freeSavingIndicator();"
                                    maxlength="2048"
                                    placeholder="Enter question text"
                                    onkeydown={ "onkeydownMCAnswer(this, event, " (item_idx) ");"}
                                    value=(item.question);
                                button
                                    title="Delete item"
                                    ."group size-5 text-red-500 tracking-tight hover:text-red-700 disabled:text-red-300 transition"
                                    hx-delete={ "/poll/item/" (item_idx) }
                                    hx-select="#pollEditingArea"
                                    hx-target="#pollEditingArea"
                                    hx-swap="outerHTML"
                                    disabled[poll.items.len() == 1 && matches!(poll.items[0].answers, Answers::Untyped)]
                                {
                                    ."block group-[.htmx-request]:hidden" { (SvgIcon::X.render()) }
                                    ."hidden size-4 group-[.htmx-request]:block" { (SvgIcon::Spinner.render()) }
                                }
                            }
                            @match &item.answers {
                                Answers::Untyped => {
                                    ."mb-2 text-slate-500 tracking-tight text-center" {
                                        "Choose item type:"
                                    }
                                    ."flex gap-4 justify-center" {
                                        button
                                            hx-post={ "/poll/item/type/" (item_idx) "/single_choice" }
                                            hx-select="#pollEditingArea"
                                            hx-target="#pollEditingArea"
                                            hx-swap="outerHTML"
                                            ."group flex justify-center items-center px-3.5 py-2 text-slate-600 border rounded hover:bg-slate-100 transition"
                                        {
                                            ."flex group-[.htmx-request]:hidden items-center justify-center gap-2" {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                                                "Multiple choice"
                                            }
                                            ."hidden size-4 group-[.htmx-request]:block" { (SvgIcon::Spinner.render()) }
                                        }
                                        button
                                            hx-post={ "/poll/item/type/" (item_idx) "/free_text" }
                                            hx-select="#pollEditingArea"
                                            hx-target="#pollEditingArea"
                                            hx-swap="outerHTML"
                                            ."group flex justify-center items-center px-3.5 py-2 text-slate-600 border rounded hover:bg-slate-100 transition"
                                        {
                                            ."flex group-[.htmx-request]:hidden items-center justify-center gap-2" {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                                "Free text"
                                            }
                                            ."hidden size-4 group-[.htmx-request]:block" { (SvgIcon::Spinner.render()) }
                                        }
                                    }
                                }
                                Answers::SingleChoice(answers) => {
                                    @let mc_answers_div_name = format!("mc-answers-div-{}", item_idx);
                                    #(mc_answers_div_name) ."flex flex-col gap-2" {
                                        @for (answer_idx, (answer_txt, is_correct)) in answers.iter().enumerate() {
                                            (render_mc_answer(item_idx, answer_idx, answer_txt, *is_correct, false))
                                        }

                                        @if answers.len() < POLL_MAX_MC_ANSWERS {
                                            button #{ "btn-add-answer-" (item_idx) }
                                                ."relative group w-fit ml-2 mb-4 text-sm text-slate-500 underline hover:text-slate-800"
                                                hx-post={ "/poll/item/" (item_idx) "/add_mc_answer" }
                                                hx-swap="beforebegin"
                                                "hx-on::after-request"={ "maybeHideAddAnswerBtn('" (mc_answers_div_name) "');" }
                                            {
                                                ."group-[.htmx-request]:opacity-0" { "Add answer" }
                                                ."absolute inset-0 size-full hidden group-[.htmx-request]:flex justify-center items-center" {
                                                    ."size-4" { (SvgIcon::Spinner.render()) }
                                                }
                                            }
                                        }
                                    }
                                },
                                Answers::FreeText(_, _) => {
                                    ."pl-2 flex gap-2 items-center text-slate-500" {
                                        ."size-4 shrink-0" { (SvgIcon::Edit3.render()) }
                                        "Free text: Participants can submit their own answer."
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button ."group mb-4 flex justify-start items-center gap-3"
               hx-post="/poll/item"
               hx-select="#pollEditingArea"
               hx-target="#pollEditingArea"
               hx-swap="outerHTML" {
                ."relative size-7 rounded-full bg-slate-600 group-hover:bg-slate-800 group-focus-within:bg-slate-800" {
                    ."absolute inset-0 size-full flex justify-center items-center text-slate-50" {
                        ."size-4" { (SvgIcon::Plus.render()) }
                    }
                }
                ."text-slate-600 group-hover:text-slate-700" {
                    "Add item"
                }
            }
            form #poll_upload_form
                ."mb-4"
                hx-post="/poll/json"
                hx-trigger="change"
                hx-encoding="multipart/form-data"
                hx-select="#pollEditingArea"
                hx-target="#pollEditingArea"
                hx-swap="outerHTML"
            {
                label
                    ."group flex justify-start items-center gap-3 cursor-pointer"
                {
                    ."relative size-7 rounded-full bg-slate-600 group-hover:bg-slate-800 group-focus-within:bg-slate-800" {
                        ."absolute inset-0 size-full flex justify-center items-center text-slate-50" {
                            ."size-4" { (SvgIcon::UploadCloud.render()) }
                        }
                    }
                    ."text-slate-600 group-hover:text-slate-700 group-focus-within:hidden" {
                        "Import poll (.json)"
                    }
                    input
                        ."hidden"
                        type="file"
                        name="poll_file"
                        accept="application/json"
                    ;
                }
            }
            a ."group mb-4 flex justify-start items-center gap-3"
                href="/poll/json"
                download="poll.json"
            {
                ."relative size-7 rounded-full bg-slate-600 group-hover:bg-slate-800 group-focus-within:bg-slate-800" {
                    ."absolute inset-0 size-full flex justify-center items-center text-slate-50" {
                        ."size-4" { (SvgIcon::Download.render()) }
                    }
                }
                ."text-slate-600 group-hover:text-slate-700 group-focus-within:hidden" {
                    "Save poll"
                }
            }
            ."mb-12 text-slate-400 text-sm max-w-xl" {
                "Your poll will be available from this device for up to 30 days. "
                "If you wish to reuse it in the future or transfer it to another device, download a copy via 'Save poll'."
            }
            ."mb-3 text-xl font-semibold text-slate-700" {
                "Options"
            }
            ."flex items-center gap-2" {
                input
                    #"check-enable-leaderboard"
                    type="checkbox"
                    name="enable_leaderboard"
                    value="true"
                    checked[poll.leaderboard_enabled]
                    hx-post="/poll/enable_leaderboard"
                    hx-swap="none"
                    hx-trigger="change"
                    ."size-4 accent-indigo-500";
                label
                    for="check-enable-leaderboard"
                    ."text-slate-600 font-semibold"
                {
                    "Leaderboard"
                }
            }
            ."ml-6 mb-3 text-slate-400 text-sm" {
                "Participants will receive points for submitting the correct answer. Longer response times will yield less points."
            }
            ."flex items-center gap-2" {
                input
                    #"check-allow-custom-names"
                    type="checkbox"
                    name="allow_custom_names"
                    value="true"
                    checked[poll.allow_custom_names]
                    hx-post="/poll/allow_custom_player_names"
                    hx-swap="none"
                    hx-trigger="change"
                    ."size-4 accent-indigo-500";
                label
                    for="check-allow-custom-names"
                    ."text-slate-600 font-semibold"
                {
                    "Custom names"
                }
            }
            ."ml-6 mb-3 text-slate-400 text-sm" {
                "Allow participants to set a custom name."
            }
        }
    }, true).into_response());
}

pub async fn post_add_item(session: Session) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

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

fn render_mc_answer(
    item_idx: usize,
    answer_idx: usize,
    answer_txt: &str,
    is_correct: bool,
    autofocus: bool,
) -> Markup {
    html! {
        ."pl-2 flex items-center gap-2 text-sm" {
            div ."transition"
                ."text-slate-500"[!is_correct]
                ."text-green-600"[is_correct] {
                    ((b'A' + answer_idx as u8) as char)
            }
            input type="text"
                name="answer_text"
                ."px-2 py-0.5 flex-1 font-medium transition"
                ."text-slate-700"[!is_correct]
                ."text-green-600"[is_correct]
                hx-put={ "/poll/item/" (item_idx) "/mc_answer/" (answer_idx) "/text" }
                hx-trigger="input changed delay:300ms"
                "hx-on::before-request"="bindSavingIndicator();"
                "hx-on::after-request"="freeSavingIndicator();"
                maxlength="2048"
                placeholder={ "Answer " (answer_idx + 1) }
                value=(answer_txt)
                onkeydown={ "onkeydownMCAnswer(this, event, " (item_idx) ");"}
                autofocus[autofocus];
            /*button
                title="Mark/Unmark answer as correct"
                ."size-5 hover:text-green-600 transition"
                ."text-slate-400"[!is_correct]
                ."text-green-600"[is_correct]
                hx-put={ "/poll/item/" (item_idx) "/mc_answer/" (answer_idx) "/toggle_correct" }
                hx-select="#pollEditingArea"
                hx-target="#pollEditingArea"
                hx-swap="outerHTML"
            {
                (svg_icons::get("check-square"))
            }*/
            button
                title="Delete answer"
                ."group delete-mc-btn size-5 flex items-center justify-center text-slate-400 hover:text-red-500 disabled:hover:text-slate-400 transition"
                hx-delete={ "/poll/item/" (item_idx) "/mc_answer/" (answer_idx) }
                hx-select="#pollEditingArea"
                hx-target="#pollEditingArea"
                hx-swap="outerHTML"
                onclick="document.querySelectorAll('.delete-mc-btn').forEach((btn) => { btn.disabled = true; })"
            {
                ."size-5 group-[.htmx-request]:hidden" { (SvgIcon::Trash2.render()) }
                ."size-4 hidden group-[.htmx-request]:block" { (SvgIcon::Spinner.render()) }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PutQuestionText {
    pub question: String,
}

pub async fn put_question_text(
    Path(item_idx): Path<usize>,
    session: Session,
    Form(form_data): Form<PutQuestionText>,
) -> Result<Response, AppError> {
    if form_data.question.chars().count() > 2048 {
        return Err(AppError::BadRequest(
            "Question text length limit of 2048 chars exceeded".to_string(),
        ));
    }

    let mut poll = PollV1::from_session(&session).await?;

    match poll.items.get_mut(item_idx) {
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
    Path((item_idx, answer_idx)): Path<(usize, usize)>,
    session: Session,
    Form(form_data): Form<PutMCAnswerForm>,
) -> Result<Response, AppError> {
    if form_data.answer_text.chars().count() > 2048 {
        return Err(AppError::BadRequest(
            "Answer text exceeds upper limit of 2048 characters".to_string(),
        ));
    }

    let mut poll = PollV1::from_session(&session).await?;

    match &mut poll
        .items
        .get_mut(item_idx)
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
    Path((item_idx, answer_idx)): Path<(usize, usize)>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

    match &mut poll
        .items
        .get_mut(item_idx)
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
    Path((item_idx, item_type_descriptor)): Path<(usize, SmartString<Compact>)>,
) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

    match poll.items.get_mut(item_idx) {
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
        None => return Err(AppError::BadRequest("item_idx out of bounds".to_string())),
    }

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

pub async fn delete_item(
    Path(item_idx): Path<usize>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

    if item_idx >= poll.items.len() {
        return Err(AppError::BadRequest("Item index out of bounds".to_string()));
    }

    poll.items.remove(item_idx);

    poll.save_to_session(&session).await?;

    return get_poll_page(session).await;
}

pub async fn post_add_mc_answer(
    Path(item_idx): Path<usize>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

    let new_answer_idx = match &mut poll
        .items
        .get_mut(item_idx)
        .ok_or(AppError::BadRequest("Item index out of bounds".to_string()))?
        .answers
    {
        Answers::SingleChoice(answers) => {
            if answers.len() >= POLL_MAX_MC_ANSWERS {
                return Err(AppError::BadRequest(
                    "Number of MC-Answers Limit already reached.".to_string(),
                ));
            }

            answers.push((String::new(), false));
            answers.len() - 1usize
        }
        _ => {
            return Err(AppError::BadRequest(
                "This is not a single choice item".to_string(),
            ));
        }
    };

    poll.save_to_session(&session).await?;

    return Ok(render_mc_answer(item_idx, new_answer_idx, "", false, true).into_response());
}

pub async fn delete_mc_answer(
    Path((item_idx, answer_idx)): Path<(usize, usize)>,
    session: Session,
) -> Result<Response, AppError> {
    let mut poll = PollV1::from_session(&session).await?;

    match &mut poll
        .items
        .get_mut(item_idx)
        .ok_or(AppError::BadRequest("Item index out of bounds".to_string()))?
        .answers
    {
        Answers::SingleChoice(answers) => {
            if answer_idx >= answers.len() {
                return Err(AppError::BadRequest(
                    "Answer index out of bounds".to_string(),
                ));
            }

            answers.remove(answer_idx);
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

pub async fn get_poll_json(session: Session) -> Result<Json<PollV1>, AppError> {
    return Ok(Json(PollV1::from_session(&session).await?));
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

    let poll: PollV1 =
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
    let mut poll = PollV1::from_session(&session).await?;
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
    let mut poll = PollV1::from_session(&session).await?;
    poll.allow_custom_names = params.allow_custom_names.is_some();
    poll.save_to_session(&session).await?;

    return Ok("Updated custom player names setting".into_response());
}
